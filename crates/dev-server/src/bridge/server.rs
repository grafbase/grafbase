use super::consts::{CREATE_TABLE, DB_FILE, DB_URL};
use super::types::{Payload, Record};
use crate::errors::DevServerError;
use axum::Extension;
use axum::{http::StatusCode, routing::post, Json, Router};
use sqlx::query::{Query, QueryAs};
use sqlx::{migrate::MigrateDatabase, query, query_as, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::fs;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

async fn query_endpoint(
    Json(payload): Json<Payload>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<Vec<Record>>, DevServerError> {
    let template = query_as::<_, Record>(&payload.query);

    let result = payload
        .iter_variables()
        .fold(template, QueryAs::bind)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(Json(result))
}

async fn mutation_endpoint(
    Json(payload): Json<Payload>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<StatusCode, DevServerError> {
    let template = query(&payload.query);

    payload
        .iter_variables()
        .fold(template, Query::bind)
        .execute(pool.as_ref())
        .await?;

    Ok(StatusCode::OK)
}

#[tokio::main]
async fn bridge_main(port: u16) -> Result<(), DevServerError> {
    if fs::metadata(DB_FILE).is_err() {
        Sqlite::create_database(DB_URL).await?;
    }

    let pool = SqlitePoolOptions::new().connect(DB_URL).await?;

    query(CREATE_TABLE).execute(&pool).await?;

    let pool = Arc::new(pool);

    let router = Router::new()
        .route("/query", post(query_endpoint))
        .route("/mutation", post(mutation_endpoint))
        .layer(Extension(pool));

    let socket_address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    axum::Server::bind(&socket_address)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

pub fn start(port: u16) -> Result<(), DevServerError> {
    bridge_main(port)
}
