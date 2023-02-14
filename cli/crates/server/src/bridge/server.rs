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
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tower_http::trace::TraceLayer;

async fn query_endpoint(
    State((_resolvers_path, _environment_variables, pool)): State<(PathBuf, HashMap<String, String>, Arc<SqlitePool>)>,
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
    State((_resolvers_path, _environment_variables, pool)): State<(PathBuf, HashMap<String, String>, Arc<SqlitePool>)>,
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
    State((resolvers_path, environment_variables, _pool)): State<(PathBuf, HashMap<String, String>, Arc<SqlitePool>)>,
    Json(payload): Json<ResolverInvocation>,
) -> Result<Json<serde_json::Value>, ApiError> {
    trace!("resolver invocation\n\n{:#?}\n", payload);
    super::resolvers::invoke_resolver(&resolvers_path, &payload.resolver_name, &environment_variables)
        .await
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

    let environment_variables =
        crate::environment::environment_variables().collect::<std::collections::HashMap<_, _>>();

    let router = Router::new()
        .route("/query", post(query_endpoint))
        .route("/mutation", post(mutation_endpoint))
        .route("/invoke-resolver", post(invoke_resolver_endpoint))
        .with_state((environment.resolvers_path(), environment_variables, Arc::clone(&pool)))
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
