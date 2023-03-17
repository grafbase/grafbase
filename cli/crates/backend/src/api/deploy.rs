use super::client::create_client;
use super::consts::{API_URL, PROJECT_METADATA_FILE};
use super::errors::ApiError;
use super::graphql::mutations::{DeploymentCreate, DeploymentCreateArguments, DeploymentCreateInput};
use super::types::ProjectMetadata;
use common::environment::Environment;
use cynic::http::ReqwestExt;
use cynic::{Id, MutationBuilder};
use tokio::fs::read_to_string;

/// # Errors
pub async fn deploy() -> Result<(), ApiError> {
    let environment = Environment::get();

    let project_metadata_file_path = environment.project_dot_grafbase_path.join(PROJECT_METADATA_FILE);

    match project_metadata_file_path.try_exists() {
        Ok(true) => {}
        Ok(false) => return Err(ApiError::UnlinkedProject),
        Err(error) => return Err(ApiError::ReadProjectMetadataFile(error)),
    }

    let project_metadata_file = read_to_string(project_metadata_file_path)
        .await
        .map_err(ApiError::ReadProjectMetadataFile)?;

    let project_metadata: ProjectMetadata =
        serde_json::from_str(&project_metadata_file).map_err(|_| ApiError::CorruptProjectMetadataFile)?;

    tokio::task::spawn_blocking(|| {
        let (tar_file, tar_file_path) = tempfile::NamedTempFile::new().unwrap().into_parts();

        let mut tar = tar::Builder::new(tar_file);
        tar.mode(tar::HeaderMode::Deterministic);

        //
        // if let Some(top_package_json_path) = None {
        //     tar.append_path_with_name(top_package_json_path, "package.json")
        //         .unwrap();
        // }
        tar.append_dir_all("grafbase", &environment.project_grafbase_path)
            .unwrap();

        tar_file_path //OK
    })
    .await
    .unwrap();

    let client = create_client().await?;

    // create archive and get size

    let operation = DeploymentCreate::build(DeploymentCreateArguments {
        input: DeploymentCreateInput {
            // TODO change to actual archive size
            archive_file_size: 500,
            // TODO remove this when it is defaulted by the API
            branch: "main".to_owned(),
            project_id: Id::new(project_metadata.project_id),
        },
    });

    let response = client.post(API_URL).run_graphql(operation).await?;

    let payload = response
        .data
        .ok_or(ApiError::UnauthorizedOrDeletedUser)?
        .deployment_create;

    // upload archive

    Ok(())
}
