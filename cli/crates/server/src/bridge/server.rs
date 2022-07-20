use super::consts::{CREATE_TABLE, DB_FILE, DB_URL_PREFIX};
use super::types::{Payload, Record};
use crate::errors::ServerError;
use crate::event::{wait_for_event, Event};
use axum::{http::StatusCode, routing::post, Extension, Json, Router};
use common::environment::Environment;
use sqlx::query::{Query, QueryAs};
use sqlx::{migrate::MigrateDatabase, query, query_as, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tower_http::trace::TraceLayer;

async fn query_endpoint(
    Json(payload): Json<Payload>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<Vec<Record>>, ServerError> {
    trace!("request\n\n{:#?}\n", payload);

    let template = query_as::<_, Record>(&payload.query);

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
    Json(payload): Json<Payload>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<StatusCode, ServerError> {
    trace!("request\n\n{:#?}\n", payload);

    let template = query(&payload.query);

    payload
        .iter_variables()
        .fold(template, Query::bind)
        .execute(pool.as_ref())
        .await
        .map_err(|error| {
            error!("mutation error: {error}");
            error
        })?;

    Ok(StatusCode::OK)
}

pub async fn start(port: u16, event_bus: Sender<Event>) -> Result<(), ServerError> {
    trace!("starting bridge at port {port}");

    let environment = Environment::get();
    let db_file = environment.project_dot_grafbase_path.join(DB_FILE);

    let db_url = match db_file.to_str() {
        Some(db_file) => format!("{DB_URL_PREFIX}{db_file}"),
        None => return Err(ServerError::ProjectPath),
    };

    if !Sqlite::database_exists(&db_url).await? {
        trace!("creating SQLite database");
        Sqlite::create_database(&db_url).await?;
    }

    let pool = SqlitePoolOptions::new().connect(&db_url).await?;

    query(CREATE_TABLE).execute(&pool).await?;

    let pool = Arc::new(pool);

    let router = Router::new()
        .route("/query", post(query_endpoint))
        .route("/mutation", post(mutation_endpoint))
        .layer(Extension(Arc::clone(&pool)))
        .layer(TraceLayer::new_for_http());

    let socket_address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    let server = axum::Server::bind(&socket_address)
        .serve(router.into_make_service())
        .with_graceful_shutdown(wait_for_event(event_bus.subscribe(), Event::Reload));

    event_bus.send(Event::BridgeReady).expect("cannot fail");

    server.await?;

    pool.close().await;

    Ok(())
}
