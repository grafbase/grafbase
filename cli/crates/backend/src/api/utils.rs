use super::{consts::PROJECT_METADATA_FILE, errors::ApiError};
use common::environment::Environment;
use tokio::fs;

pub async fn project_linked() -> Result<bool, ApiError> {
    let environment = Environment::get();
    fs::try_exists(environment.project_dot_grafbase_path.join(PROJECT_METADATA_FILE))
        .await
        .map_err(ApiError::ReadProjectMetadataFile)
}
