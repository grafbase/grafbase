use super::{consts::PROJECT_METADATA_FILE, errors::ApiError};
use common::environment::Project;
use tokio::fs;

pub async fn project_linked() -> Result<bool, ApiError> {
    let project = Project::get();
    fs::try_exists(project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE))
        .await
        .map_err(ApiError::ReadProjectMetadataFile)
}
