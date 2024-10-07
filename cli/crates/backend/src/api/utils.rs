use super::errors::ApiError;
use common::{consts::PROJECT_METADATA_FILE, environment::Project};
use tokio::fs;

pub async fn has_project_linked() -> Result<bool, ApiError> {
    let project = Project::get();
    fs::try_exists(project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE))
        .await
        .map_err(ApiError::ReadProjectMetadataFile)
}
