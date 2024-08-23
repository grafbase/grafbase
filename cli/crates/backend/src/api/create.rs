use super::client::create_client;
use super::consts::api_url;
use super::deploy;
use super::errors::{ApiError, CreateError};
use super::graphql::mutations::{
    CurrentPlanLimitReachedError, DuplicateDatabaseRegionsError, EnvironmentVariableSpecification, GraphCreate,
    GraphCreateArguments, GraphCreateInput, GraphCreatePayload, InvalidDatabaseRegionsError, SlugTooLongError,
};
use super::graphql::queries::viewer_for_create::{PersonalAccount, Viewer};
use super::types::{Account, ProjectMetadata};
use common::consts::PROJECT_METADATA_FILE;
use common::environment::Project;
use cynic::http::ReqwestExt;
use cynic::Id;
use cynic::{MutationBuilder, QueryBuilder};
use std::iter;
use tokio::fs;

pub use super::graphql::mutations::GraphMode;

/// # Errors
///
/// See [`ApiError`]
pub async fn get_viewer_data_for_creation() -> Result<Vec<Account>, ApiError> {
    let client = create_client().await?;
    let query = Viewer::build(());
    let response = client.post(api_url()).run_graphql(query).await?;
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
    graph_mode: GraphMode,
    env_vars: impl Iterator<Item = (&str, &str)>,
) -> Result<(Vec<String>, Option<cynic::Id>, String), ApiError> {
    let project = if let GraphMode::Managed = graph_mode {
        let project = Project::get();

        match project.dot_grafbase_directory_path.try_exists() {
            Ok(true) => {}
            Ok(false) => fs::create_dir_all(&project.dot_grafbase_directory_path)
                .await
                .map_err(ApiError::CreateProjectDotGrafbaseFolder)?,
            Err(error) => return Err(ApiError::ReadProjectDotGrafbaseFolder(error)),
        }

        Some(project)
    } else {
        None
    };

    let client = create_client().await?;

    let operation = GraphCreate::build(GraphCreateArguments {
        input: GraphCreateInput {
            account_id: Id::new(account_id),
            graph_slug: project_slug,
            graph_mode,
            repo_root_path: project.map(|project| {
                project
                    .schema_path
                    .path()
                    .parent()
                    .expect("must have a parent")
                    .strip_prefix(&project.path)
                    .expect("must be a prefix")
                    .to_str()
                    .expect("must be a valid string")
            }),
            environment_variables: env_vars
                .map(|(name, value)| EnvironmentVariableSpecification { name, value })
                .collect(),
        },
    });

    let response = client.post(api_url()).run_graphql(operation).await?;
    let payload = response.data.ok_or(ApiError::UnauthorizedOrDeletedUser)?.graph_create;

    match payload {
        GraphCreatePayload::GraphCreateSuccess(graph_create_success) => {
            if let Some(project) = project {
                let project_metadata_path = project.dot_grafbase_directory_path.join(PROJECT_METADATA_FILE);

                tokio::fs::write(
                    &project_metadata_path,
                    ProjectMetadata::new(graph_create_success.graph.id.into_inner().clone()).to_string(),
                )
                .await
                .map_err(ApiError::WriteProjectMetadataFile)?;
            }

            let deployment_id = if matches!(graph_mode, GraphMode::Managed) {
                let (deployment_id, _, _) = deploy::deploy(None, None).await?;
                Some(deployment_id)
            } else {
                None
            };

            let domains = graph_create_success
                .graph
                .production_branch
                .domains
                .iter()
                .map(|domain| format!("{domain}/graphql"))
                .collect();

            Ok((domains, deployment_id, graph_create_success.graph.slug))
        }
        GraphCreatePayload::SlugAlreadyExistsError(_) => Err(CreateError::SlugAlreadyExists.into()),
        GraphCreatePayload::SlugInvalidError(_) => Err(CreateError::SlugInvalid.into()),
        GraphCreatePayload::SlugTooLongError(SlugTooLongError { max_length, .. }) => {
            Err(CreateError::SlugTooLong { max_length }.into())
        }
        GraphCreatePayload::AccountDoesNotExistError(_) => Err(CreateError::AccountDoesNotExist.into()),
        GraphCreatePayload::CurrentPlanLimitReachedError(CurrentPlanLimitReachedError { max, .. }) => {
            Err(CreateError::CurrentPlanLimitReached { max }.into())
        }
        GraphCreatePayload::DuplicateDatabaseRegionsError(DuplicateDatabaseRegionsError { duplicates, .. }) => {
            Err(CreateError::DuplicateDatabaseRegions { duplicates }.into())
        }
        GraphCreatePayload::EmptyDatabaseRegionsError(_) => Err(CreateError::EmptyDatabaseRegions.into()),
        GraphCreatePayload::InvalidDatabaseRegionsError(InvalidDatabaseRegionsError { invalid, .. }) => {
            Err(CreateError::InvalidDatabaseRegions { invalid }.into())
        }
        GraphCreatePayload::InvalidEnvironmentVariablesError(_) => Err(CreateError::InvalidEnvironmentVariables.into()),
        GraphCreatePayload::EnvironmentVariableCountLimitExceededError(_) => {
            Err(CreateError::EnvironmentVariableCountLimitExceeded.into())
        }
        GraphCreatePayload::DisabledAccountError(_) => Err(CreateError::DisabledAccount.into()),
        GraphCreatePayload::Unknown(error) => Err(CreateError::Unknown(error).into()),
    }
}
