use crate::consts::{API_URL, CREDENTIALS_FILE, PROJECT_METADATA_FILE, USER_AGENT};
use crate::errors::CreateApiError;
use crate::graphql::mutations::{
    CurrentPlanLimitReachedError, DuplicateDatabaseRegionsError, InvalidDatabaseRegionsError, ProjectCreate,
    ProjectCreateArguments, ProjectCreateInput, ProjectCreatePayload, ProjectCreateSuccess, SlugTooLongError,
};
use crate::graphql::queries;
use crate::types::{Credentials, DatabaseRegion, ProjectMetadata};
use crate::{errors::BackendError, graphql::queries::PersonalAccount, types::Account};
use common::environment::{get_user_dot_grafbase_path, Environment};
use cynic::Id;
use cynic::{http::ReqwestExt, MutationBuilder, QueryBuilder};
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Client;
use std::iter;
use tokio::fs::read_to_string;

async fn get_authenticated_client() -> Result<reqwest::Client, BackendError> {
    // needed to bypass the project fallback behavior of Environment's dot grafbase folder
    // TODO consider removing the fallback
    let user_dot_grafbase_path = get_user_dot_grafbase_path().ok_or(BackendError::FindUserDotGrafbaseFolder)?;

    match user_dot_grafbase_path.try_exists() {
        Ok(true) => {}
        Ok(false) => return Err(BackendError::LoggedOut),
        Err(error) => return Err(BackendError::ReadUserDotGrafbaseFolder(error)),
    }

    let credentials_file_path = user_dot_grafbase_path.join(CREDENTIALS_FILE);

    match credentials_file_path.try_exists() {
        Ok(true) => {}
        Ok(false) => return Err(BackendError::LoggedOut),
        Err(error) => return Err(BackendError::ReadCredentialsFile(error)),
    }

    let credential_file = read_to_string(user_dot_grafbase_path.join(CREDENTIALS_FILE))
        .await
        .map_err(BackendError::ReadCredentialsFile)?;

    let credentials: Credentials<'_> =
        serde_json::from_str(&credential_file).map_err(|_| BackendError::CorruptCredentialsFile)?;

    let token = credentials.access_token;

    let mut headers = HeaderMap::new();
    let mut bearer_token =
        HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| BackendError::CorruptCredentialsFile)?;
    bearer_token.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, bearer_token);
    let mut user_agent = HeaderValue::from_str(USER_AGENT).expect("must be visible ascii");
    user_agent.set_sensitive(true);
    headers.insert(header::USER_AGENT, user_agent);

    Ok(Client::builder()
        .default_headers(headers)
        .build()
        .expect("TLS is supported in all targets"))
}

/// # Errors
/// # Panics
pub async fn get_viewer_data_for_creation() -> Result<(Vec<Account>, String), BackendError> {
    // TODO consider if we want to do this elsewhere
    if project_linked() {
        return Err(BackendError::ProjectAlreadyLinked);
    }

    let client = get_authenticated_client().await?;

    let query = queries::ViewerAndClosestRegion::build(());

    let response = client.post(API_URL).run_graphql(query).await?;

    let response = response.data.expect("must exist");

    let viewer_response = response.viewer.ok_or(BackendError::UnauthorizedOrDeletedUser)?;

    let closest_region = response
        .closest_database_region
        .ok_or(BackendError::UnauthorizedOrDeletedUser)?
        .name;

    let PersonalAccount { id, name, slug } = viewer_response
        .personal_account
        .ok_or(BackendError::IncorrectlyScopedToken)?;

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

    Ok((accounts, closest_region))
}

/// # Errors
pub async fn create(
    account_id: &str,
    project_slug: &str,
    database_regions: &[DatabaseRegion],
) -> Result<Vec<String>, BackendError> {
    let environment = Environment::get();

    let client = get_authenticated_client().await?;

    let operation = ProjectCreate::build(ProjectCreateArguments {
        input: ProjectCreateInput {
            account_id: Id::new(account_id),
            database_regions: database_regions.iter().map(ToString::to_string).collect(),
            project_slug: project_slug.to_owned(),
        },
    });

    let response = client.post(API_URL).run_graphql(operation).await?;

    let payload = response
        .data
        .ok_or(BackendError::UnauthorizedOrDeletedUser)?
        .project_create;

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
            .map_err(BackendError::WriteProjectMetadataFile)?;
            Ok(project.production_branch.domains)
        }
        ProjectCreatePayload::SlugAlreadyExistsError(_) => Err(CreateApiError::SlugAlreadyExists.into()),
        ProjectCreatePayload::SlugInvalidError(_) => Err(CreateApiError::SlugInvalid.into()),
        ProjectCreatePayload::SlugTooLongError(SlugTooLongError { max_length, .. }) => {
            Err(CreateApiError::SlugTooLong { max_length }.into())
        }
        ProjectCreatePayload::AccountDoesNotExistError(_) => Err(CreateApiError::AccountDoesNotExist.into()),
        ProjectCreatePayload::CurrentPlanLimitReachedError(CurrentPlanLimitReachedError { max, .. }) => {
            Err(CreateApiError::CurrentPlanLimitReached { max }.into())
        }
        ProjectCreatePayload::DuplicateDatabaseRegionsError(DuplicateDatabaseRegionsError { duplicates, .. }) => {
            Err(CreateApiError::DuplicateDatabaseRegions { duplicates }.into())
        }
        ProjectCreatePayload::EmptyDatabaseRegionsError(_) => Err(CreateApiError::EmptyDatabaseRegions.into()),
        ProjectCreatePayload::InvalidDatabaseRegionsError(InvalidDatabaseRegionsError { invalid, .. }) => {
            Err(CreateApiError::InvalidDatabaseRegions { invalid }.into())
        }
        ProjectCreatePayload::Unknown => Err(CreateApiError::Unknown.into()),
    }
}

fn project_linked() -> bool {
    let environment = Environment::get();
    environment
        .project_dot_grafbase_path
        .join(PROJECT_METADATA_FILE)
        .exists()
}
