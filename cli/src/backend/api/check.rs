pub use super::graphql::mutations::{SchemaCheck, SchemaCheckErrorSeverity, SchemaCheckGitCommitInput};

use super::{
    client::create_client,
    errors::ApiError,
    graphql::mutations::{SchemaCheckCreate, SchemaCheckCreateArguments, SchemaCheckCreateInput, SchemaCheckPayload},
};
use crate::common::environment::PlatformData;
use cynic::{http::ReqwestExt, MutationBuilder};

pub enum SchemaCheckResult {
    Ok(SchemaCheck),
    SubgraphNameMissingOnFederatedGraphError,
}

pub async fn check(
    account: &str,
    graph: &str,
    branch: Option<&str>,
    subgraph_name: &str,
    schema: &str,
    git_commit: Option<SchemaCheckGitCommitInput>,
) -> Result<SchemaCheckResult, ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client().await?;

    let operation = SchemaCheckCreate::build(SchemaCheckCreateArguments {
        input: SchemaCheckCreateInput {
            account_slug: account,
            graph_slug: Some(graph),
            branch,
            subgraph_name: Some(subgraph_name),
            schema,
            git_commit,
        },
    });

    let result = client.post(&platform_data.api_url).run_graphql(operation).await?;

    match result {
        cynic::GraphQlResponse {
            data:
                Some(SchemaCheckCreate {
                    schema_check_create: Some(SchemaCheckPayload::SubgraphNameMissingOnFederatedGraphError(_)),
                }),
            errors: _,
        } => Ok(SchemaCheckResult::SubgraphNameMissingOnFederatedGraphError),
        cynic::GraphQlResponse {
            data:
                Some(SchemaCheckCreate {
                    schema_check_create: Some(SchemaCheckPayload::SchemaCheck(sc)),
                }),
            errors: _,
        } => Ok(SchemaCheckResult::Ok(sc)),
        _ => Err(ApiError::RequestError(format!("API error:\n\n{result:#?}",))),
    }
}
