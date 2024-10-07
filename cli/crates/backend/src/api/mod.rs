use common::consts::PROJECT_METADATA_FILE;

use self::{errors::ApiError, types::ProjectMetadata};

pub mod graphql;
mod utils;

pub mod branch;
pub mod check;
pub mod client;
pub mod consts;
pub mod create;
pub mod errors;
pub mod link;
pub mod login;
pub mod logout;
pub mod publish;
pub mod schema;
pub mod subgraphs;
pub mod submit_trusted_documents;
pub mod types;
pub mod unlink;

pub(crate) fn project_metadata() -> Result<ProjectMetadata, ApiError> {
    let project = common::environment::Project::get();
    let project_metadata_file_path = project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE);

    match project_metadata_file_path.try_exists() {
        Ok(true) => {}
        Ok(false) => return Err(ApiError::UnlinkedProject),
        Err(error) => return Err(ApiError::ReadProjectMetadataFile(error)),
    }

    let project_metadata_file =
        std::fs::read_to_string(project_metadata_file_path).map_err(ApiError::ReadProjectMetadataFile)?;

    let result = serde_json::from_str(&project_metadata_file).map_err(|_| ApiError::CorruptProjectMetadataFile)?;

    Ok(result)
}
