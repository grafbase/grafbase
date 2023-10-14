use super::{consts::PROJECT_METADATA_FILE, errors::ApiError, types::ProjectMetadata};
use common::environment::Project;
use tokio::fs;

pub async fn project_linked() -> Result<Option<ProjectMetadata>, ApiError> {
    let project = Project::get();

    let project_metadata_file_path = project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE);

    if !fs::try_exists(&project_metadata_file_path)
        .await
        .map_err(ApiError::ReadProjectMetadataFile)?
    {
        return Ok(None);
    }

    let project_metadata_file = tokio::fs::read_to_string(project_metadata_file_path)
        .await
        .map_err(ApiError::ReadProjectMetadataFile)?;

    Ok(Some(
        serde_json::from_str(&project_metadata_file).map_err(|_| ApiError::CorruptProjectMetadataFile)?,
    ))
}

pub async fn has_project_linked() -> Result<bool, ApiError> {
    let project = Project::get();
    fs::try_exists(project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE))
        .await
        .map_err(ApiError::ReadProjectMetadataFile)
}
