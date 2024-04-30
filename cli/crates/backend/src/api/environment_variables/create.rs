use cynic::{http::ReqwestExt, MutationBuilder};

use crate::api::{
    client::create_client,
    consts::api_url,
    errors::ApiError,
    graphql::mutations::environment_variable_upsert_by_slugs::{
        BranchEnvironment, EnvironmentVariableUpsert, EnvironmentVariableUpsertBySlugs,
        EnvironmentVariableUpsertBySlugsVariables, EnvironmentVariableUpsertVariables,
    },
};

pub(super) async fn with_slugs(
    account_slug: &str,
    project_slug: &str,
    name: &str,
    value: &str,
    branch_environment: impl IntoIterator<Item = &str>,
) -> Result<(), ApiError> {
    let client = create_client().await?;

    let environments = branch_environment
        .into_iter()
        .flat_map(|env| {
            if env == "all" {
                vec![BranchEnvironment::Preview, BranchEnvironment::Production]
            } else if env == "production" {
                vec![BranchEnvironment::Production]
            } else {
                vec![BranchEnvironment::Preview]
            }
        })
        .collect();

    let operation = EnvironmentVariableUpsertBySlugs::build(EnvironmentVariableUpsertBySlugsVariables {
        account_slug,
        project_slug,
        environments,
        name,
        value,
    });

    let cynic::GraphQlResponse { errors, .. } = client.post(api_url()).run_graphql(operation).await?;

    if let Some(errors) = errors {
        return Err(ApiError::RequestError(format!("{errors:#?}")));
    }

    Ok(())
}

pub(super) async fn with_linked(
    name: &str,
    value: &str,
    branch_environment: impl IntoIterator<Item = &str>,
) -> Result<(), ApiError> {
    let project_metadata = crate::api::project_metadata()?;
    let client = create_client().await?;

    let environments = branch_environment
        .into_iter()
        .flat_map(|env| {
            if env == "all" {
                vec![BranchEnvironment::Preview, BranchEnvironment::Production]
            } else if env == "production" {
                vec![BranchEnvironment::Production]
            } else {
                vec![BranchEnvironment::Preview]
            }
        })
        .collect();

    let operation = EnvironmentVariableUpsert::build(EnvironmentVariableUpsertVariables {
        project_id: project_metadata.project_id.into(),
        environments,
        name,
        value,
    });

    let cynic::GraphQlResponse { errors, .. } = client.post(api_url()).run_graphql(operation).await?;

    if let Some(errors) = errors {
        return Err(ApiError::RequestError(format!("{errors:#?}")));
    }

    Ok(())
}
