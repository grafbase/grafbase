use cynic::{http::ReqwestExt, GraphQlError, MutationBuilder};

use crate::api::{
    client::create_client,
    consts::api_url,
    errors::{ApiError, EnvironmentVariableError},
    graphql::mutations::{
        environment_variable_delete::{
            EnvironmentVariableDeleteByValuesPayload, EnvironmentVariableDeleteWithValues,
            EnvironmentVariableDeleteWithValuesBySlug, EnvironmentVariableDeleteWithValuesBySlugVariables,
            EnvironmentVariableDeleteWithValuesVariables,
        },
        environment_variable_upsert_by_slugs::BranchEnvironment,
    },
};

pub(super) async fn with_slugs(
    account_slug: &str,
    project_slug: &str,
    name: &str,
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

    let operation =
        EnvironmentVariableDeleteWithValuesBySlug::build(EnvironmentVariableDeleteWithValuesBySlugVariables {
            account_slug,
            project_slug,
            environments,
            name,
        });

    let cynic::GraphQlResponse { errors, data } = client.post(api_url()).run_graphql(operation).await?;
    let result = data.map(|v| v.environment_variable_delete_with_values_by_slug);

    handle_result(name, result, errors)
}

pub(super) async fn with_linked(
    name: &str,
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

    let operation = EnvironmentVariableDeleteWithValues::build(EnvironmentVariableDeleteWithValuesVariables {
        project_id: project_metadata.graph_id().into(),
        environments,
        name,
    });

    let cynic::GraphQlResponse { errors, data } = client.post(api_url()).run_graphql(operation).await?;
    let result = data.map(|v| v.environment_variable_delete_with_values);

    handle_result(name, result, errors)
}

fn handle_result(
    name: &str,
    results: Option<EnvironmentVariableDeleteByValuesPayload>,
    errors: Option<Vec<GraphQlError>>,
) -> Result<(), ApiError> {
    match results {
        Some(EnvironmentVariableDeleteByValuesPayload::EnvironmentVariableDeleteByValuesSuccess(_)) => Ok(()),
        Some(EnvironmentVariableDeleteByValuesPayload::EnvironmentVariableDoesNotExistError(_)) => {
            Err(EnvironmentVariableError::NotFound(name.to_string()).into())
        }
        Some(EnvironmentVariableDeleteByValuesPayload::Unknown(error)) => {
            Err(EnvironmentVariableError::Unknown(error).into())
        }
        None => match errors {
            Some(errors) => Err(ApiError::RequestError(format!("{errors:#?}"))),
            None => Ok(()),
        },
    }
}
