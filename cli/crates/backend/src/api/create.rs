use super::client::create_client;
use super::consts::{API_URL, PROJECT_METADATA_FILE};
use super::deploy;
use super::errors::{ApiError, CreateError};
use super::graphql::mutations::{
    CurrentPlanLimitReachedError, DuplicateDatabaseRegionsError, InvalidDatabaseRegionsError, ProjectCreate,
    ProjectCreateArguments, ProjectCreateInput, ProjectCreatePayload, SlugTooLongError,
};
use super::graphql::queries::viewer_for_create::{PersonalAccount, Viewer};
use super::types::{Account, ProjectMetadata};
use super::utils::has_project_linked;
use common::environment::Project;
use cynic::http::ReqwestExt;
use cynic::Id;
use cynic::{MutationBuilder, QueryBuilder};
use std::iter;
use tokio::fs;

/// # Errors
///
/// See [`ApiError`]
pub async fn get_viewer_data_for_creation() -> Result<Vec<Account>, ApiError> {
    // TODO consider if we want to do this elsewhere
    if has_project_linked().await? {
        return Err(ApiError::ProjectAlreadyLinked);
    }

    let client = create_client().await?;

    let query = Viewer::build(());

    let response = client.post(API_URL).run_graphql(query).await?;

    let response = response.data.expect("must exist");

    let viewer_response = response.viewer.ok_or(ApiError::UnauthorizedOrDeletedUser)?;

    let PersonalAccount { id, name, slug } = viewer_response
        .personal_account
        .ok_or(ApiError::IncorrectlyScopedToken)?;

    let personal_account_id = id;

    let personal_account = Account {
        id: personal_account_id.inner().to_owned(),
        name,
        slug,
        personal: true,
    };

    let accounts = iter::once(personal_account)
        .chain(viewer_response.organizations.nodes.iter().map(|organization| Account {
            id: organization.id.inner().to_owned(),
            name: organization.name.clone(),
            slug: organization.slug.clone(),
            personal: false,
        }))
        .collect();

    Ok(accounts)
}

/// # Errors
///
/// See [`ApiError`]
pub async fn create(
    account_id: &str,
    project_slug: &str,
    database_regions: &[String],
) -> Result<Vec<String>, ApiError> {
    let project = Project::get();

    match project.dot_grafbase_directory_path.try_exists() {
        Ok(true) => {}
        Ok(false) => fs::create_dir_all(&project.dot_grafbase_directory_path)
            .await
            .map_err(ApiError::CreateProjectDotGrafbaseFolder)?,
        Err(error) => return Err(ApiError::ReadProjectDotGrafbaseFolder(error)),
    }

    let client = create_client().await?;

    let operation = ProjectCreate::build(ProjectCreateArguments {
        input: ProjectCreateInput {
            account_id: Id::new(account_id),
            database_regions,
            project_slug,
            project_root_path: project
                .schema_path
                .path()
                .parent()
                .expect("must have a parent")
                .strip_prefix(&project.path)
                .expect("must be a prefix")
                .to_str()
                .expect("must be a valid string"),
        },
    });

    let response = client.post(API_URL).run_graphql(operation).await?;

    let payload = response.data.ok_or(ApiError::UnauthorizedOrDeletedUser)?.project_create;

    match payload {
        ProjectCreatePayload::ProjectCreateSuccess(project_create_success) => {
            let project_metadata_path = project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE);

            tokio::fs::write(
                &project_metadata_path,
                ProjectMetadata {
                    project_id: project_create_success.project.id.into_inner().clone(),
                }
                .to_string(),
            )
            .await
            .map_err(ApiError::WriteProjectMetadataFile)?;

            deploy::deploy().await?;

            let domains = project_create_success
                .project
                .production_branch
                .domains
                .iter()
                .map(|domain| format!("{domain}/graphql"))
                .collect();

            Ok(domains)
        }
        ProjectCreatePayload::SlugAlreadyExistsError(_) => Err(CreateError::SlugAlreadyExists.into()),
        ProjectCreatePayload::SlugInvalidError(_) => Err(CreateError::SlugInvalid.into()),
        ProjectCreatePayload::SlugTooLongError(SlugTooLongError { max_length, .. }) => {
            Err(CreateError::SlugTooLong { max_length }.into())
        }
        ProjectCreatePayload::AccountDoesNotExistError(_) => Err(CreateError::AccountDoesNotExist.into()),
        ProjectCreatePayload::CurrentPlanLimitReachedError(CurrentPlanLimitReachedError { max, .. }) => {
            Err(CreateError::CurrentPlanLimitReached { max }.into())
        }
        ProjectCreatePayload::DuplicateDatabaseRegionsError(DuplicateDatabaseRegionsError { duplicates, .. }) => {
            Err(CreateError::DuplicateDatabaseRegions { duplicates }.into())
        }
        ProjectCreatePayload::EmptyDatabaseRegionsError(_) => Err(CreateError::EmptyDatabaseRegions.into()),
        ProjectCreatePayload::InvalidDatabaseRegionsError(InvalidDatabaseRegionsError { invalid, .. }) => {
            Err(CreateError::InvalidDatabaseRegions { invalid }.into())
        }
        ProjectCreatePayload::InvalidEnvironmentVariablesError(_) => {
            Err(CreateError::InvalidEnvironmentVariables.into())
        }
        ProjectCreatePayload::EnvironmentVariableCountLimitExceededError(_) => {
            Err(CreateError::EnvironmentVariableCountLimitExceeded.into())
        }
        ProjectCreatePayload::DisabledAccountError(_) => Err(CreateError::DisabledAccount.into()),
        ProjectCreatePayload::Unknown(error) => Err(CreateError::Unknown(error).into()),
    }
}
