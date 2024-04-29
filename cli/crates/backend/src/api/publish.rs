use super::{
    client::create_client,
    consts::api_url,
    errors::{ApiError, PublishError},
    graphql::mutations::{
        FederatedGraphCompositionError, PublishPayload, SchemaRegistryBranchDoesNotExistError, SubgraphCreateArguments,
        SubgraphPublish,
    },
};
use cynic::{http::ReqwestExt, MutationBuilder};

pub enum PublishOutcome {
    Success { composition_errors: Vec<String> },
    ProjectDoesNotExist { account_name: String, project_name: String },
}

pub async fn publish(
    account_slug: &str,
    project_slug: &str,
    branch: Option<&str>,
    subgraph_name: &str,
    url: &str,
    schema: &str,
) -> Result<PublishOutcome, ApiError> {
    let client = create_client().await?;

    let operation = SubgraphPublish::build(SubgraphCreateArguments {
        input: super::graphql::mutations::PublishInput {
            account_slug,
            project_slug,
            branch,
            subgraph: subgraph_name,
            url,
            schema,
        },
    });

    let cynic::GraphQlResponse { data, errors } = client.post(api_url()).run_graphql(operation).await?;

    if let Some(data) = data {
        match data.publish {
            PublishPayload::ProjectDoesNotExistError(_) => Ok(PublishOutcome::ProjectDoesNotExist {
                account_name: account_slug.to_owned(),
                project_name: project_slug.to_owned(),
            }),
            PublishPayload::PublishSuccess(_) => Ok(PublishOutcome::Success {
                composition_errors: vec![],
            }),
            PublishPayload::FederatedGraphCompositionError(FederatedGraphCompositionError {
                messages: composition_errors,
            }) => Ok(PublishOutcome::Success { composition_errors }),
            PublishPayload::BranchDoesNotExistError(SchemaRegistryBranchDoesNotExistError { .. }) => {
                Err(ApiError::PublishError(PublishError::BranchDoesNotExist))
            }
            PublishPayload::Unknown(unknown_variant) => {
                Err(ApiError::PublishError(PublishError::Unknown(unknown_variant)))
            }
        }
    } else {
        Err(ApiError::RequestError(format!("{errors:#?}")))
    }
}
