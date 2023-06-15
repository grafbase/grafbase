use super::api_counterfeit::registry::Registry;
use super::api_counterfeit::search::{QueryExecutionRequest, QueryExecutionResponse};
use super::consts::{DATABASE_FILE, DATABASE_URL_PREFIX, PREPARE};
use super::search::Index;
use super::types::{Mutation, Operation, Record, ResolverInvocation};
use crate::bridge::api_counterfeit::registry::VersionedRegistry;
use crate::bridge::errors::ApiError;
use crate::bridge::listener;
use crate::bridge::types::{Constraint, ConstraintKind, OperationKind};
use crate::errors::ServerError;
use crate::event::{wait_for_event, Event};
use crate::types::ServerMessage;
use axum::extract::State;
use axum::Json;
use axum::{http::StatusCode, routing::post, Router};
use common::environment::{Environment, Project};

use sqlx::query::{Query, QueryAs};
use sqlx::{migrate::MigrateDatabase, query, query_as, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use tokio::fs;
use tokio::sync::{Mutex, Notify};

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use tower_http::trace::TraceLayer;

enum ResolverBuild {
    InProgress {
        notify: Arc<Notify>,
    },
    Succeeded {
        #[allow(dead_code)]
        miniflare_handle: tokio::task::JoinHandle<()>,
        worker_port: u16,
    },
    Failed,
}

struct HandlerState {
    worker_port: u16,
    pool: SqlitePool,
    bridge_sender: tokio::sync::mpsc::Sender<ServerMessage>,
    resolver_builds: Mutex<std::collections::HashMap<String, ResolverBuild>>,
    environment_variables: HashMap<String, String>,
    tracing: bool,
}

async fn query_endpoint(
    State(handler_state): State<Arc<HandlerState>>,
    Json(payload): Json<Operation>,
) -> Result<Json<Vec<Record>>, ApiError> {
    trace!("request\n\n{:#?}\n", payload);

    let template = query_as::<_, Record>(&payload.sql);

    let result = payload
        .iter_variables()
        .fold(template, QueryAs::bind)
        .fetch_all(&handler_state.as_ref().pool)
        .await
        .map_err(|error| {
            error!("query error: {error}");
            error
        })?;

    trace!("response\n\n{:#?}\n", result);

    Ok(Json(result))
}

async fn mutation_endpoint(
    State(handler_state): State<Arc<HandlerState>>,
    Json(payload): Json<Mutation>,
) -> Result<StatusCode, ApiError> {
    trace!("request\n\n{:#?}\n", payload);

    if payload.mutations.is_empty() {
        return Ok(StatusCode::OK);
    };

    let mut transaction = handler_state.as_ref().pool.begin().await.map_err(|error| {
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

async fn search_endpoint(
    State(handler_state): State<Arc<HandlerState>>,
    Json(request): Json<QueryExecutionRequest>,
) -> Result<Json<QueryExecutionResponse>, ApiError> {
    let project = Project::get();

    let registry: Registry = {
        let path = &project.registry_path;
        let file = File::open(path).map_err(|err| {
            error!("Failed to open {path:?}: {err:?}");
            ApiError::ServerError
        })?;
        let reader = BufReader::new(file);
        let versioned: VersionedRegistry = serde_json::from_reader(reader).map_err(|err| {
            error!("Failed to deserialize registry: {err:?}");
            ApiError::ServerError
        })?;
        versioned.registry
    };

    let response = Index::build(&handler_state.pool, &request.entity_type, &registry.search_config)
        .await?
        .search(request.query, request.pagination)?;
    Ok(Json(response))
}

#[allow(clippy::too_many_lines)]
async fn invoke_resolver_endpoint(
    State(handler_state): State<Arc<HandlerState>>,
    Json(payload): Json<ResolverInvocation>,
) -> Result<Json<serde_json::Value>, ApiError> {
    trace!("resolver invocation\n\n{:#?}\n", payload);

    let environment = Environment::get();

    let resolver_worker_port = loop {
        let mut resolver_builds = handler_state.resolver_builds.lock().await;

        let notify = if let Some(resolver_build) = resolver_builds.get(&payload.resolver_name) {
            match resolver_build {
                ResolverBuild::Succeeded { worker_port, .. } => break *worker_port,
                ResolverBuild::Failed => return Err(ApiError::ResolverSpawnError),
                ResolverBuild::InProgress { notify } => {
                    if Arc::strong_count(notify) == 1 {
                        let notify = notify.clone();
                        resolver_builds.remove(&payload.resolver_name);
                        notify
                    } else {
                        let notify = notify.clone();
                        drop(resolver_builds);
                        notify.notified().await;
                        continue;
                    }
                }
            }
        } else {
            Arc::new(Notify::new())
        };

        resolver_builds.insert(
            payload.resolver_name.clone(),
            ResolverBuild::InProgress { notify: notify.clone() },
        );
        let start = std::time::Instant::now();
        handler_state
            .bridge_sender
            .send(ServerMessage::StartResolverBuild(payload.resolver_name.clone()))
            .await
            .unwrap();

        let tracing = handler_state.tracing;
        if let Ok((package_json_path, wrangler_toml_path)) = crate::custom_resolvers::build_resolver(
            environment,
            environment.project.as_ref().expect("must be present"),
            &handler_state.environment_variables,
            &payload.resolver_name,
            tracing,
        )
        .await
        {
            let (miniflare_handle, worker_port) = super::resolvers::spawn_miniflare(
                &payload.resolver_name,
                handler_state.worker_port,
                package_json_path,
                wrangler_toml_path,
                tracing,
            )
            .await?;

            resolver_builds.insert(
                payload.resolver_name.clone(),
                ResolverBuild::Succeeded {
                    miniflare_handle,
                    worker_port,
                },
            );
            notify.notify_waiters();

            handler_state
                .bridge_sender
                .send(ServerMessage::CompleteResolverBuild {
                    name: payload.resolver_name.clone(),
                    duration: start.elapsed(),
                })
                .await
                .unwrap();

            break worker_port;
        }

        resolver_builds.insert(payload.resolver_name.clone(), ResolverBuild::Failed);
        notify.notify_waiters();
        return Err(ApiError::ResolverSpawnError);
    };

    super::resolvers::invoke_resolver(
        &handler_state.bridge_sender,
        resolver_worker_port,
        &payload.resolver_name,
        &payload.payload,
    )
    .await
    .map_err(|_| ApiError::ResolverInvalid(payload.resolver_name.clone()))
    .map(Json)
}

pub async fn start(
    port: u16,
    worker_port: u16,
    bridge_sender: tokio::sync::mpsc::Sender<ServerMessage>,
    event_bus: tokio::sync::broadcast::Sender<Event>,
    tracing: bool,
) -> Result<(), ServerError> {
    trace!("starting bridge at port {port}");

    let project = Project::get();

    let environment_variables: std::collections::HashMap<_, _> = crate::environment::variables().collect();

    match project.database_directory_path.try_exists() {
        Ok(true) => {}
        Ok(false) => fs::create_dir_all(&project.database_directory_path)
            .await
            .map_err(ServerError::CreateDatabaseDir)?,
        Err(error) => return Err(ServerError::ReadDatabaseDir(error)),
    }

    let database_file = project.database_directory_path.join(DATABASE_FILE);

    let db_url = match database_file.to_str() {
        Some(db_file) => format!("{DATABASE_URL_PREFIX}{db_file}"),
        None => return Err(ServerError::ProjectPath),
    };

    if !Sqlite::database_exists(&db_url).await? {
        trace!("creating SQLite database");
        Sqlite::create_database(&db_url).await?;
    }

    let pool = SqlitePoolOptions::new().connect(&db_url).await?;

    query(PREPARE).execute(&pool).await?;

    let handler_state = Arc::new(HandlerState {
        worker_port,
        pool,
        bridge_sender,
        resolver_builds: Mutex::default(),
        environment_variables,
        tracing,
    });

    let router = Router::new()
        .route("/query", post(query_endpoint))
        .route("/mutation", post(mutation_endpoint))
        .route("/search", post(search_endpoint))
        .route("/invoke-resolver", post(invoke_resolver_endpoint))
        .with_state(handler_state.clone())
        .layer(TraceLayer::new_for_http());

    let socket_address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    let server = axum::Server::bind(&socket_address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(wait_for_event(event_bus.subscribe(), |event| {
            event.should_restart_servers()
        }));

    event_bus.send(Event::BridgeReady).expect("cannot fail");

    tokio::select! {
        server_result = server => { server_result? }
        listener_result = listener::start(worker_port, event_bus.clone()) => { listener_result? }
    };

    handler_state.pool.close().await;

    Ok(())
}
