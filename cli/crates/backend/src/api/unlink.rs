use super::errors::ApiError;
use common::consts::PROJECT_METADATA_FILE;
use common::environment::Project;
use std::fs;

/// # Errors
///
/// - returns [`BackendError::UnlinkedProject`] if the project is not linked
///
/// - returns [`BackendError::DeleteProjectMetadataFile`] if ~/.grafbase/project.json could not be deleted
///
/// - returns [`BackendError::ReadProjectMetadataFile`] if ~/.grafbase/project.json could not be read
pub fn unlink() -> Result<(), ApiError> {
    let project = Project::get();

    let project_metadata_path = project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE);

    match project_metadata_path.try_exists() {
        Ok(true) => fs::remove_file(project_metadata_path).map_err(ApiError::DeleteProjectMetadataFile),
        Ok(false) => Err(ApiError::UnlinkedProject),
        Err(error) => Err(ApiError::ReadProjectMetadataFile(error)),
    }
}
