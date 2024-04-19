use super::client::create_client;
use super::consts::{api_url, PACKAGE_JSON, TAR_CONTENT_TYPE};
use super::errors::{ApiError, DeployError};
use super::graphql::mutations::{
    ArchiveFileSizeLimitExceededError, DailyDeploymentCountLimitExceededError, DeploymentCreate,
    DeploymentCreateArguments, DeploymentCreateInput, DeploymentCreatePayload,
};
use super::types::ProjectMetadata;
use common::consts::{PROJECT_METADATA_FILE, USER_AGENT};
use common::environment::Project;
use cynic::http::ReqwestExt;
use cynic::{Id, MutationBuilder};
use ignore::DirEntry;
use reqwest::{header, Body, Client};
use std::ffi::OsStr;
use std::path::Path;
use tokio::fs::read_to_string;
use tokio_util::codec::{BytesCodec, FramedRead};

const ENTRY_BLACKLIST: [&str; 2] = ["node_modules", ".env"];

/// # Errors
///
/// See [`ApiError`]
pub async fn deploy(branch: Option<&str>) -> Result<(), ApiError> {
    let project = Project::get();

    let project_metadata_file_path = project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE);

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

    let tar_file_path = tokio::task::spawn_blocking(|| {
        let (tar_file, tar_file_path) = tempfile::NamedTempFile::new()
            .map_err(ApiError::CreateTempFile)?
            .into_parts();

        let mut tar = tar::Builder::new(tar_file);
        tar.mode(tar::HeaderMode::Deterministic);

        if project.path.join(PACKAGE_JSON).exists() {
            tar.append_path_with_name(project.path.join(PACKAGE_JSON), PACKAGE_JSON)
                .map_err(ApiError::AppendToArchive)?;
        }

        let walker = ignore::WalkBuilder::new(&project.path)
            .hidden(false)
            .filter_entry(|entry| should_traverse_entry(entry, &project.path))
            .build();

        for entry in walker {
            let entry = entry.map_err(ApiError::ReadProjectFile)?;

            if entry.path() == project.path {
                // walker always includes the root path in its traversal, so skip that
                continue;
            }

            let entry_path = entry.path().to_owned();
            let path_in_tar = entry_path.strip_prefix(&project.path).expect("must include prefix");
            let entry_metadata = entry.metadata().map_err(ApiError::ReadProjectFile)?;
            if entry_metadata.is_file() {
                tar.append_path_with_name(&entry_path, path_in_tar)
                    .map_err(ApiError::AppendToArchive)?;
            } else {
                // as we don't follow links, anything else will be a directory
                tar.append_dir(path_in_tar, &entry_path)
                    .map_err(ApiError::AppendToArchive)?;
            }
        }

        tar.finish().map_err(ApiError::WriteArchive)?;

        Result::Ok::<_, ApiError>(tar_file_path)
    })
    .await
    .expect("must be fine")?;

    let tar_file = tokio::fs::File::open(&tar_file_path)
        .await
        .map_err(ApiError::ReadArchive)?;

    let client = create_client().await?;

    #[allow(clippy::cast_possible_truncation)] // must fit or will be over allowed limit
    let archive_file_size = tar_file.metadata().await.map_err(ApiError::ReadArchiveMetadata)?.len() as i32;

    let operation = DeploymentCreate::build(DeploymentCreateArguments {
        input: DeploymentCreateInput {
            archive_file_size,
            branch,
            project_id: Id::new(project_metadata.project_id),
        },
    });

    let response = client.post(api_url()).run_graphql(operation).await?;

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

            Ok(())
        }
        DeploymentCreatePayload::ProjectDoesNotExistError(_) => Err(DeployError::ProjectDoesNotExist.into()),
        DeploymentCreatePayload::ArchiveFileSizeLimitExceededError(ArchiveFileSizeLimitExceededError {
            limit, ..
        }) => Err(DeployError::ArchiveFileSizeLimitExceeded { limit }.into()),
        DeploymentCreatePayload::DailyDeploymentCountLimitExceededError(DailyDeploymentCountLimitExceededError {
            limit,
            ..
        }) => Err(DeployError::DailyDeploymentCountLimitExceeded { limit }.into()),
        DeploymentCreatePayload::Unknown(error) => Err(DeployError::Unknown(error).into()),
    }
}

fn should_traverse_entry(entry: &DirEntry, root_path: &Path) -> bool {
    entry
        .path()
        .strip_prefix(root_path)
        .expect("must contain the project directory")
        .file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|entry_name| !ENTRY_BLACKLIST.contains(&entry_name))
}
