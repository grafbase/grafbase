use super::consts::{DB_FILE, DB_URL_PREFIX, PREPARE};
use super::types::{Mutation, Operation, Record, ResolverInvocation};
use crate::bridge::errors::ApiError;
use crate::bridge::listener;
use crate::bridge::types::{Constraint, ConstraintKind, OperationKind};
use crate::errors::ServerError;
use crate::event::{wait_for_event, Event};
use axum::extract::State;
use axum::Json;
use axum::{http::StatusCode, routing::post, Router};
use common::environment::Environment;
use sqlx::query::{Query, QueryAs};
use sqlx::{migrate::MigrateDatabase, query, query_as, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tower_http::trace::TraceLayer;

async fn query_endpoint(
    State((_resolvers_path, pool)): State<(PathBuf, Arc<SqlitePool>)>,
    Json(payload): Json<Operation>,
) -> Result<Json<Vec<Record>>, ApiError> {
    trace!("request\n\n{:#?}\n", payload);

    let template = query_as::<_, Record>(&payload.sql);

    let result = payload
        .iter_variables()
        .fold(template, QueryAs::bind)
        .fetch_all(pool.as_ref())
        .await
        .map_err(|error| {
            error!("query error: {error}");
            error
        })?;

    trace!("response\n\n{:#?}\n", result);

    Ok(Json(result))
}

async fn mutation_endpoint(
    State((_resolvers_path, pool)): State<(PathBuf, Arc<SqlitePool>)>,
    Json(payload): Json<Mutation>,
) -> Result<StatusCode, ApiError> {
    trace!("request\n\n{:#?}\n", payload);

    if payload.mutations.is_empty() {
        return Ok(StatusCode::OK);
    };

    let mut transaction = pool.begin().await.map_err(|error| {
        error!("transaction start error: {error}");
        error
    })?;

    for operation in payload.mutations {
        let template = query(&operation.sql);

        let query = operation.iter_variables().fold(template, Query::bind);

        query.execute(&mut transaction).await.map_err(|error| {
            error!("mutation error: {error}");
            match operation.kind {
                Some(OperationKind::Constraint(Constraint {
                    kind: ConstraintKind::Unique,
                    ..
                })) => ApiError::from_error_and_operation(error, operation),
                None => error.into(),
            }
        })?;
    }

    transaction.commit().await.map_err(|error| {
        error!("transaction commit error: {error}");
        error
    })?;

    Ok(StatusCode::OK)
}

async fn invoke_resolver_endpoint(
    State((resolvers_path, _pool)): State<(PathBuf, Arc<SqlitePool>)>,
    Json(payload): Json<ResolverInvocation>,
) -> Result<Json<serde_json::Value>, ApiError> {
    trace!("resolver invocation\n\n{:#?}\n", payload);

    let resolver_source_code =
        tokio::fs::read_to_string(resolvers_path.join(&payload.resolver_name).with_extension("js"))
            .await
            .map_err(|_| ApiError::ResolverDoesNotExist(payload.resolver_name.clone()))?;

    let isolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    let resource_name = v8::String::new(scope, &payload.resolver_name).unwrap();
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);
    let code = v8::String::new(scope, &resolver_source_code).unwrap();
    let source_map_url = v8::null(scope).into();
    let origin = v8::ScriptOrigin::new(
        scope,
        resource_name.into(),
        0,
        0,
        false,
        0,
        source_map_url,
        false,
        false,
        true,
    );

    trace!("instantiating the module");

    let source = v8::script_compiler::Source::new(code, Some(&origin));
    let return_value = {
        let tc_scope = &mut v8::TryCatch::new(scope);

        let module = v8::script_compiler::compile_module(tc_scope, source).ok_or_else(|| {
            let error = tc_scope.exception().unwrap().to_rust_string_lossy(tc_scope);
            error!("v8 error: {error}");
            ApiError::ResolverInvalid(payload.resolver_name.clone())
        })?;
        trace!("module compiled");

        module
            .instantiate_module(tc_scope, |_context, _string, _fixed_array, module| Some(module))
            .ok_or_else(|| {
                let error = tc_scope.exception().unwrap().to_rust_string_lossy(tc_scope);
                error!("v8 error: {error}");
                ApiError::ResolverInvalid(payload.resolver_name.clone())
            })?;
        trace!("module instantiated");

        let _ = module.evaluate(tc_scope);
        let module_namespace: v8::Local<'_, v8::Object> = module.get_module_namespace().try_into().unwrap();
        let default_key = v8::String::new(tc_scope, "default").unwrap();
        let module_namespace = module_namespace.get(tc_scope, default_key.into()).unwrap();
        let default_function: v8::Local<'_, v8::Function> = module_namespace.try_into().map_err(|error| {
            error!("v8 error: {error}");
            ApiError::ResolverInvalid(payload.resolver_name.clone())
        })?;

        let global = context.global(tc_scope).into();
        trace!("about to run the exported function");

        let context = v8::Object::new(tc_scope).into();
        let return_value = default_function.call(tc_scope, global, &[context]).ok_or_else(|| {
            let error = tc_scope.exception().unwrap().to_rust_string_lossy(tc_scope);
            error!("v8 error: {error}");
            ApiError::ResolverInvalid(payload.resolver_name.clone())
        })?;
        serde_v8::from_v8(tc_scope, return_value)
    };

    return_value
        .map_err(|_| ApiError::ResolverInvalid(payload.resolver_name.clone()))
        .map(Json)
}

pub async fn start(port: u16, worker_port: u16, event_bus: Sender<Event>) -> Result<(), ServerError> {
    trace!("starting bridge at port {port}");

    let environment = Environment::get();
    let db_file = environment.project_dot_grafbase_path().join(DB_FILE);

    let db_url = match db_file.to_str() {
        Some(db_file) => format!("{DB_URL_PREFIX}{db_file}"),
        None => return Err(ServerError::ProjectPath),
    };

    if !Sqlite::database_exists(&db_url).await? {
        trace!("creating SQLite database");
        Sqlite::create_database(&db_url).await?;
    }

    let pool = SqlitePoolOptions::new().connect(&db_url).await?;

    query(PREPARE).execute(&pool).await?;

    let pool = Arc::new(pool);

    let router = Router::new()
        .route("/query", post(query_endpoint))
        .route("/mutation", post(mutation_endpoint))
        .route("/invoke-resolver", post(invoke_resolver_endpoint))
        .with_state((environment.resolvers_path(), Arc::clone(&pool)))
        .layer(TraceLayer::new_for_http());

    let socket_address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    let server = axum::Server::bind(&socket_address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(wait_for_event(event_bus.subscribe(), Event::Reload));

    event_bus.send(Event::BridgeReady).expect("cannot fail");

    tokio::select! {
        server_result = server => { server_result? }
        listener_result = listener::start(worker_port, event_bus) => { listener_result? }
    };

    pool.close().await;

    Ok(())
}
