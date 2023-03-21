use super::client::create_client;
use super::consts::{API_URL, PROJECT_METADATA_FILE};
use super::deploy;
use super::errors::{ApiError, CreateError};
use super::graphql::mutations::{
    CurrentPlanLimitReachedError, DuplicateDatabaseRegionsError, InvalidDatabaseRegionsError, ProjectCreate,
    ProjectCreateArguments, ProjectCreateInput, ProjectCreatePayload, ProjectCreateSuccess, SlugTooLongError,
};
use super::graphql::queries::{self, PersonalAccount};
use super::types::{Account, DatabaseRegion, ProjectMetadata};
use common::environment::Environment;
use cynic::http::ReqwestExt;
use cynic::Id;
use cynic::{MutationBuilder, QueryBuilder};
use std::iter;

/// # Errors
/// # Panics
pub async fn get_viewer_data_for_creation() -> Result<(Vec<Account>, Vec<DatabaseRegion>, DatabaseRegion), ApiError> {
    // TODO consider if we want to do this elsewhere
    if project_linked() {
        return Err(ApiError::ProjectAlreadyLinked);
    }

    let client = create_client().await?;

    let query = queries::ViewerAndRegions::build(());

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
        .chain(
            viewer_response
                .organization_memberships
                .iter()
                .map(|membership| Account {
                    id: membership.account.id.inner().to_owned(),
                    name: membership.account.name.clone(),
                    slug: membership.account.slug.clone(),
                    personal: false,
                }),
        )
        .collect();

    Ok((accounts, available_regions, closest_region))
}

/// # Errors
pub async fn create(
    account_id: &str,
    project_slug: &str,
    database_regions: &[DatabaseRegion],
) -> Result<Vec<String>, ApiError> {
    let environment = Environment::get();

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
            // TODO prevent reset from deleting this
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

fn project_linked() -> bool {
    let environment = Environment::get();
    environment
        .project_dot_grafbase_path
        .join(PROJECT_METADATA_FILE)
        .exists()
}
