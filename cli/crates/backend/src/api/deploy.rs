use super::client::create_client;
use super::consts::{API_URL, GRAFBASE_DIR_NAME, PACKAGE_JSON, PROJECT_METADATA_FILE};
use super::errors::ApiError;
use super::graphql::mutations::{
    DeploymentCreate, DeploymentCreateArguments, DeploymentCreateInput, DeploymentCreatePayload,
};
use super::types::ProjectMetadata;
use common::environment::Environment;
use cynic::http::ReqwestExt;
use cynic::{Id, MutationBuilder};
use reqwest::{header, Body, Client};
use tokio::fs::read_to_string;
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_util::compat::TokioAsyncReadCompatExt;

/// # Errors
/// # Panics
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

    // ERROR
    let (tar_file, tar_file_path) = tempfile::NamedTempFile::new().unwrap().into_parts();

    let tar_file: tokio::fs::File = tar_file.into();
    let tar_file = tar_file.compat();

    let mut tar = async_tar::Builder::new(tar_file);
    tar.mode(async_tar::HeaderMode::Deterministic);

    if environment.project_path.join(PACKAGE_JSON).exists() {
        tar.append_path_with_name(environment.project_path.join(PACKAGE_JSON), PACKAGE_JSON)
            .await
            .unwrap();
    }

    // ERROR
    tar.append_dir_all(GRAFBASE_DIR_NAME, &environment.project_grafbase_path)
        .await
        .unwrap();

    let tar_file = tokio::fs::File::open(&tar_file_path).await.unwrap();

    let client = create_client().await?;

    let content_length = tar_file.metadata().await.unwrap().len() as i32; // must fit or will be over allowed limit

    let operation = DeploymentCreate::build(DeploymentCreateArguments {
        input: DeploymentCreateInput {
            // ERROR
            archive_file_size: content_length,
            branch: None,
            project_id: Id::new(project_metadata.project_id),
        },
    });

    let response = client.post(API_URL).run_graphql(operation).await?;

    let payload = response
        .data
        .ok_or(ApiError::UnauthorizedOrDeletedUser)?
        .deployment_create;

    match payload {
        DeploymentCreatePayload::DeploymentCreateSuccess(payload) => {
            let framed_tar = FramedRead::new(tar_file, BytesCodec::new());
            let response = Client::new()
                .put(payload.presigned_url)
                .header(header::CONTENT_LENGTH, content_length)
                .header(header::CONTENT_TYPE, "application/x-tar")
                .body(Body::wrap_stream(framed_tar))
                .send()
                .await
                // ERROR
                .unwrap();

            dbg!(response.text().await);
        }
        // ERROR
        DeploymentCreatePayload::ProjectDoesNotExistError(_) => todo!(),
        DeploymentCreatePayload::ArchiveFileSizeLimitExceededError(_) => todo!(),
        DeploymentCreatePayload::DailyDeploymentCountLimitExceededError(_) => todo!(),
        DeploymentCreatePayload::Unknown => todo!(),
    }

    Ok(())
}
