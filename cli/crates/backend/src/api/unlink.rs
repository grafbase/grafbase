use super::{consts::PROJECT_METADATA_FILE, errors::ApiError};
use common::environment::Environment;
use std::fs;

/// # Errors
///
/// - returns [`BackendError::UnlinkedProject`] if the project is not linked
///
/// - returns [`BackendError::DeleteProjectMetadataFile`] if ~/.grafbase/project.json could not be deleted
///
/// - returns [`BackendError::ReadProjectMetadataFile`] if ~/.grafbase/project.json could not be read
pub fn unlink() -> Result<(), ApiError> {
    let environment = Environment::get();

    let project_metadata_path = environment.project_dot_grafbase_path.join(PROJECT_METADATA_FILE);

    match project_metadata_path.try_exists() {
        Ok(true) => fs::remove_file(project_metadata_path).map_err(ApiError::DeleteProjectMetadataFile),
        Ok(false) => Err(ApiError::UnlinkedProject),
        Err(error) => Err(ApiError::ReadProjectMetadataFile(error)),
    }
}
