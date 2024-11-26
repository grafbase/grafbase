use crate::errors::BackendError;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use federated_server::ServerRouter;
use flate2::bufread::GzDecoder;
use std::path::Path;
use tar::Archive;
use tokio::fs;
use tower_http::services::ServeDir;

const DOT_GRAFBASE_DIR: &str = ".grafbase";
const PATHFINDER_ASSETS_DIR: &str = "pathfinder";
const VERSION_FILE_NAME: &str = "version";
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn export_assets() -> Result<(), BackendError> {
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
            return Ok(());
        }
    }

    let tar = include_bytes!("../../assets/pathfinder.tar.gz");

    Archive::new(GzDecoder::new(tar.as_slice()))
        .unpack(dot_grafbase_dir.join(PATHFINDER_ASSETS_DIR))
        .map_err(BackendError::UnpackPathfinderArchive)?;

    fs::write(dot_grafbase_dir.join(VERSION_FILE_NAME), CARGO_PKG_VERSION)
        .await
        .map_err(BackendError::WriteAssetVersion)?;

    Ok(())
}

pub fn get_pathfinder_router<T>(port: u16, home_dir: &Path) -> ServerRouter<T> {
    let html =
        include_str!("pathfinder.html").replace("{{ GRAPHQL_URL }}", &format!("http://127.0.0.1:{port}/graphql"));

    Router::new()
        .route("/", get(root))
        .nest_service(
            "/static",
            ServeDir::new(home_dir.join(DOT_GRAFBASE_DIR).join(PATHFINDER_ASSETS_DIR)),
        )
        .with_state(PathfinderState { html: Html(html) })
}

#[allow(clippy::unused_async)]
async fn root(State(PathfinderState { html }): State<PathfinderState>) -> impl IntoResponse {
    html
}

#[derive(Clone)]
struct PathfinderState {
    html: Html<String>,
}
