use cynic::{http::ReqwestExt, QueryBuilder};

use crate::api::{
    client::create_client,
    consts::api_url,
    errors::ApiError,
    graphql::queries::list_env_vars::{
        ListEnvironmentVariables, ListEnvironmentVariablesArguments, ListEnvironmentVariablesBySlugs,
        ListEnvironmentVariablesBySlugsArguments, Node, Project,
    },
};

pub(super) async fn with_linked_project() -> Result<Project, ApiError> {
    let project_metadata = crate::api::project_metadata()?;

    let operation = ListEnvironmentVariables::build(ListEnvironmentVariablesArguments {
        project_id: project_metadata.project_id.into(),
    });

    let client = create_client().await?;
    let cynic::GraphQlResponse { data, errors } = client.post(api_url()).run_graphql(operation).await?;

    match (data.and_then(|d| d.node), errors) {
        (Some(Node::Project(project)), _) => Ok(project),
        (None, None) => Err(ApiError::ProjectDoesNotExist),
        (_, errors) => Err(ApiError::RequestError(format!("{errors:#?}"))),
    }
}

pub(super) async fn with_slugs(account_slug: &str, project_slug: &str) -> Result<Project, ApiError> {
    let operation = ListEnvironmentVariablesBySlugs::build(ListEnvironmentVariablesBySlugsArguments {
        account_slug,
        project_slug,
    });

    let client = create_client().await?;
    let cynic::GraphQlResponse { data, errors } = client.post(api_url()).run_graphql(operation).await?;

    match (data.and_then(|d| d.project_by_account_slug), errors) {
        (Some(project), _) => Ok(project),
        (None, None) => Err(ApiError::ProjectDoesNotExist),
        (_, errors) => Err(ApiError::RequestError(format!("{errors:#?}"))),
    }
}
