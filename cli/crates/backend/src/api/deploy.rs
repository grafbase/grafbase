use super::client::create_client;
use super::consts::{API_URL, GRAFBASE_DIR_NAME, PACKAGE_JSON, PROJECT_METADATA_FILE, TAR_CONTENT_TYPE};
use super::errors::{ApiError, DeployError};
use super::graphql::mutations::{
    ArchiveFileSizeLimitExceededError, DailyDeploymentCountLimitExceededError, DeploymentCreate,
    DeploymentCreateArguments, DeploymentCreateInput, DeploymentCreatePayload,
};
use super::types::ProjectMetadata;
use crate::consts::USER_AGENT;
use common::environment::Environment;
use cynic::http::ReqwestExt;
use cynic::{Id, MutationBuilder};
use reqwest::{header, Body, Client};
use tokio::fs::read_to_string;
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio_util::compat::TokioAsyncReadCompatExt;

/// # Errors
///
/// See [`ApiError`]
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

    let (tar_file, tar_file_path) = tempfile::NamedTempFile::new()
        .map_err(ApiError::CreateTempFile)?
        .into_parts();

    let tar_file: tokio::fs::File = tar_file.into();
    let tar_file = tar_file.compat();

    let mut tar = async_tar::Builder::new(tar_file);
    tar.mode(async_tar::HeaderMode::Deterministic);

    if environment.project_path.join(PACKAGE_JSON).exists() {
        tar.append_path_with_name(environment.project_path.join(PACKAGE_JSON), PACKAGE_JSON)
            .await
            .map_err(ApiError::AppendToArchive)?;
    }

    tar.append_dir_all(GRAFBASE_DIR_NAME, &environment.project_grafbase_path)
        .await
        .map_err(ApiError::AppendToArchive)?;

    let tar_file = tokio::fs::File::open(&tar_file_path)
        .await
        .map_err(ApiError::ReadArchive)?;

    let client = create_client().await?;

    #[allow(clippy::cast_possible_truncation)] // must fit or will be over allowed limit
    let archive_file_size = tar_file.metadata().await.map_err(ApiError::ReadArchiveMetadata)?.len() as i32;

    let operation = DeploymentCreate::build(DeploymentCreateArguments {
        input: DeploymentCreateInput {
            archive_file_size,
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
                .header(header::CONTENT_LENGTH, archive_file_size)
                .header(header::CONTENT_TYPE, TAR_CONTENT_TYPE)
                .header(header::USER_AGENT, USER_AGENT)
                .body(Body::wrap_stream(framed_tar))
                .send()
                .await
                .map_err(|_| ApiError::UploadError)?;

            if !response.status().is_success() {
                return Err(ApiError::UploadError);
            }
        }
        DeploymentCreatePayload::ProjectDoesNotExistError(_) => return Err(DeployError::ProjectDoesNotExist.into()),
        DeploymentCreatePayload::ArchiveFileSizeLimitExceededError(ArchiveFileSizeLimitExceededError {
            limit, ..
        }) => return Err(DeployError::ArchiveFileSizeLimitExceededError { limit }.into()),
        DeploymentCreatePayload::DailyDeploymentCountLimitExceededError(DailyDeploymentCountLimitExceededError {
            limit,
            ..
        }) => return Err(DeployError::DailyDeploymentCountLimitExceededError { limit }.into()),
        DeploymentCreatePayload::Unknown => return Err(DeployError::Unknown.into()),
    }

    Ok(())
}
