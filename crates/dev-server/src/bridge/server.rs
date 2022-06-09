use super::consts::{CREATE_TABLE, DB_FILE, DB_URL_PREFIX};
use super::types::{Payload, Record};
use crate::errors::DevServerError;
use axum::{http::StatusCode, routing::post, Extension, Json, Router};
use common::environment::Environment;
use sqlx::query::{Query, QueryAs};
use sqlx::{migrate::MigrateDatabase, query, query_as, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::fs;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

async fn query_endpoint(
    Json(payload): Json<Payload>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<Json<Vec<Record>>, DevServerError> {
    trace!("request\n\n{:#?}\n", payload);

    let template = query_as::<_, Record>(&payload.query);

    let result = payload
        .iter_variables()
        .fold(template, QueryAs::bind)
        .fetch_all(pool.as_ref())
        .await?;

    trace!("response\n\n{:#?}\n", result);

    Ok(Json(result))
}

async fn mutation_endpoint(
    Json(payload): Json<Payload>,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<StatusCode, DevServerError> {
    trace!("request\n\n{:#?}\n", payload);

    let template = query(&payload.query);

    payload
        .iter_variables()
        .fold(template, Query::bind)
        .execute(pool.as_ref())
        .await?;

    Ok(StatusCode::OK)
}

pub async fn start(port: u16) -> Result<(), DevServerError> {
    trace!("starting bridge at port {port}");

    let environment = Environment::get();
    let project_dot_grafbase_path = environment.project_dot_grafbase_path.clone();
    let db_file = environment.project_dot_grafbase_path.join(DB_FILE);

    let db_url = match db_file.to_str() {
        Some(db_file) => format!("{DB_URL_PREFIX}{db_file}"),
        None => return Err(DevServerError::ProjectPath),
    };

    if fs::metadata(&project_dot_grafbase_path).is_err() {
        trace!("creating .grafbase directory");
        fs::create_dir_all(&project_dot_grafbase_path).map_err(|_| DevServerError::CreateCacheDir)?;
    }

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
        .layer(Extension(pool))
        .layer(TraceLayer::new_for_http());

    let socket_address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    axum::Server::bind(&socket_address)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
