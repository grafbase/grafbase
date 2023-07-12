use super::consts::{DATABASE_FILE, DATABASE_URL_PREFIX, PREPARE};
use super::types::{Mutation, Operation, Record};
use super::udf::UdfBuild;
use crate::bridge::errors::ApiError;
use crate::bridge::listener;
use crate::bridge::log::log_event_endpoint;
use crate::bridge::search::search_endpoint;
use crate::bridge::types::{Constraint, ConstraintKind, OperationKind};
use crate::bridge::udf::invoke_udf_endpoint;
use crate::errors::ServerError;
use crate::event::{wait_for_event, Event};
use crate::types::ServerMessage;
use axum::extract::State;
use axum::Json;
use axum::{http::StatusCode, routing::post, Router};
use common::environment::Project;

use common::types::UdfKind;
use sqlx::query::{Query, QueryAs};
use sqlx::{migrate::MigrateDatabase, query, query_as, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use tokio::fs;
use tokio::sync::Mutex;

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use tower_http::trace::TraceLayer;

pub struct HandlerState {
    pub pool: SqlitePool,
    pub bridge_sender: tokio::sync::mpsc::Sender<ServerMessage>,
    pub udf_builds: Mutex<std::collections::HashMap<(String, UdfKind), UdfBuild>>,
    pub environment_variables: HashMap<String, String>,
    pub tracing: bool,
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
        pool,
        bridge_sender,
        udf_builds: Mutex::default(),
        environment_variables,
        tracing,
    });

    let router = Router::new()
        .route("/query", post(query_endpoint))
        .route("/mutation", post(mutation_endpoint))
        .route("/search", post(search_endpoint))
        .route("/invoke-udf", post(invoke_udf_endpoint))
        .route("/log-event", post(log_event_endpoint))
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
