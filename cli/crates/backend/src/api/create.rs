use super::client::create_client;
use super::consts::{API_URL, PROJECT_METADATA_FILE};
use super::deploy;
use super::errors::{ApiError, CreateError};
use super::graphql::mutations::{
    CurrentPlanLimitReachedError, DuplicateDatabaseRegionsError, InvalidDatabaseRegionsError, ProjectCreate,
    ProjectCreateArguments, ProjectCreateInput, ProjectCreatePayload, ProjectCreateSuccess, SlugTooLongError,
};
use super::graphql::queries::viewer_and_regions::{PersonalAccount, ViewerAndRegions};
use super::types::{Account, DatabaseRegion, ProjectMetadata};
use super::utils::project_linked;
use common::environment::Environment;
use cynic::http::ReqwestExt;
use cynic::Id;
use cynic::{MutationBuilder, QueryBuilder};
use std::iter;
use tokio::fs;

/// # Errors
///
/// See [`ApiError`]
pub async fn get_viewer_data_for_creation() -> Result<(Vec<Account>, Vec<DatabaseRegion>, DatabaseRegion), ApiError> {
    // TODO consider if we want to do this elsewhere
    if project_linked().await? {
        return Err(ApiError::ProjectAlreadyLinked);
    }

    let client = create_client().await?;

    let query = ViewerAndRegions::build(());

    let response = client.post(API_URL).run_graphql(query).await?;

    let response = response.data.expect("must exist");

    let viewer_response = response.viewer.ok_or(ApiError::UnauthorizedOrDeletedUser)?;

    let closest_region = response
        .closest_database_region
        .ok_or(ApiError::UnauthorizedOrDeletedUser)?
        .into();

    let available_regions = response.database_regions.into_iter().map(Into::into).collect();

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

    Ok((accounts, available_regions, closest_region))
}

/// # Errors
///
/// See [`ApiError`]
pub async fn create(
    account_id: &str,
    project_slug: &str,
    database_regions: &[String],
) -> Result<Vec<String>, ApiError> {
    let environment = Environment::get();

    match environment.project_dot_grafbase_path.try_exists() {
        Ok(true) => {}
        Ok(false) => fs::create_dir_all(&environment.project_dot_grafbase_path)
            .await
            .map_err(ApiError::CreateProjectDotGrafbaseFolder)?,
        Err(error) => return Err(ApiError::ReadProjectDotGrafbaseFolder(error)),
    }

    let client = create_client().await?;

    let operation = ProjectCreate::build(ProjectCreateArguments {
        input: ProjectCreateInput {
            account_id: Id::new(account_id),
            database_regions: database_regions.iter().map(ToString::to_string).collect(),
            project_slug: project_slug.to_owned(),
        },
    });

    let response = client.post(API_URL).run_graphql(operation).await?;

    let payload = response.data.ok_or(ApiError::UnauthorizedOrDeletedUser)?.project_create;

    match payload {
        ProjectCreatePayload::ProjectCreateSuccess(ProjectCreateSuccess { project, .. }) => {
            let project_metadata_path = environment.project_dot_grafbase_path.join(PROJECT_METADATA_FILE);

            tokio::fs::write(
                &project_metadata_path,
                ProjectMetadata {
                    account_id: account_id.to_owned(),
                    project_id: project.id.into_inner().clone(),
                }
                .to_string(),
            )
            .await
            .map_err(ApiError::WriteProjectMetadataFile)?;

            deploy::deploy().await?;

            Ok(project.production_branch.domains)
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
        ProjectCreatePayload::Unknown => Err(CreateError::Unknown.into()),
    }
}
