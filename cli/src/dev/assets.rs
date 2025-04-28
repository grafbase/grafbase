use crate::errors::BackendError;
use axum::{
    Router,
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, get_service},
};
use flate2::bufread::GzDecoder;

use std::{path::PathBuf, sync::Arc};
use tar::Archive;
use tokio::fs;
use tower_http::services::ServeDir;

use super::{SubgraphCache, data_json::DataJson};

const INDEX_FILE_NAME: &str = "index.html";
const DOT_GRAFBASE_DIR: &str = ".grafbase";
const ASSETS_DIR_NAME: &str = "cli-app";
const VERSION_FILE_NAME: &str = "version";
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn assets_dir() -> anyhow::Result<PathBuf> {
    let Some(home_dir) = dirs::home_dir() else {
        anyhow::bail!("Failed to determine home directory");
    };

    Ok(home_dir.join(DOT_GRAFBASE_DIR).join(ASSETS_DIR_NAME))
}

pub(crate) async fn export_assets() -> Result<(), BackendError> {
    let home_dir = dirs::home_dir().ok_or(BackendError::HomeDirectory)?;
    let dot_grafbase_dir = home_dir.join(DOT_GRAFBASE_DIR);
    if !dot_grafbase_dir
        .try_exists()
        .map_err(BackendError::AccessDotGrafbaseDirectory)?
    {
        fs::create_dir_all(&dot_grafbase_dir)
            .await
            .map_err(BackendError::CreateDotGrafbaseDirectory)?;
    }

    let version_file = dot_grafbase_dir.join(VERSION_FILE_NAME);

    if version_file.try_exists().map_err(BackendError::ReadAssetVersion)? {
        let version = fs::read_to_string(version_file)
            .await
            .map_err(BackendError::ReadAssetVersion)?;
        if version == CARGO_PKG_VERSION {
            tracing::info!("CLI assets already exist in the latest version. Skipping.");

            return Ok(());
        }
    }

    tracing::info!("Decompressing CLI assets");

    let tar = include_bytes!("../../assets/cli-app.tar.gz");

    Archive::new(GzDecoder::new(tar.as_slice()))
        .unpack(dot_grafbase_dir.join(ASSETS_DIR_NAME))
        .map_err(BackendError::UnpackCliAppArchive)?;

    fs::write(dot_grafbase_dir.join(VERSION_FILE_NAME), CARGO_PKG_VERSION)
        .await
        .map_err(BackendError::WriteAssetVersion)?;

    Ok(())
}

pub(crate) fn get_base_router<S>(
    graphql_url: Arc<tokio::sync::OnceCell<String>>,
    mcp_url: Option<String>,
    subgraph_cache: Arc<SubgraphCache>,
) -> Router<S> {
    let assets_dir = assets_dir().expect("assets directory exists");
    let index_path = assets_dir.join(INDEX_FILE_NAME);

    let index_html = std::fs::read_to_string(&index_path).expect("we create this a step above");

    Router::new()
        .route("/", get(root))
        .nest_service(
            "/app",
            get_service(ServeDir::new(assets_dir).fallback(tower_http::services::ServeFile::new(index_path))),
        )
        .route("/app/data.json", get(data_json))
        .with_state(AppState {
            html: Html(index_html),
            graphql_url,
            mcp_url,
            subgraph_cache,
        })
}

async fn root(State(AppState { html, .. }): State<AppState>) -> impl IntoResponse {
    html
}

async fn data_json(State(state): State<AppState>) -> axum::response::Response<axum::body::Body> {
    let response = state
        .subgraph_cache
        .with_data_json_schemas(|updated_at, schemas| {
            let data_json = DataJson {
                updated_at,
                graphql_api_url: state
                    .graphql_url
                    .get()
                    .expect("GraphQL URL should be set in data_json handler"),
                mcp_server_url: state.mcp_url.as_deref(),
                schemas,
            };

            serde_json::to_vec(&data_json)
        })
        .await;

    match response {
        Ok(response) => {
            let mut response = axum::response::Response::new(axum::body::Body::from(response));
            response.headers_mut().insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("application/json"),
            );

            response
        }
        Err(err) => {
            tracing::error!("Error serializing data.json: {err}");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Clone)]
struct AppState {
    html: Html<String>,
    graphql_url: Arc<tokio::sync::OnceCell<String>>,
    mcp_url: Option<String>,
    subgraph_cache: Arc<SubgraphCache>,
}
