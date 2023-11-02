use crate::api::graphql::queries::{
    fetch_federated_graph_schema::{FetchFederatedGraphSchemaArguments, FetchFederatedGraphSchemaQuery},
    fetch_subgraph_schema::{FetchSubgraphSchemaArguments, FetchSubgraphSchemaQuery},
};

use super::{client::create_client, consts::API_URL, errors::ApiError};
use cynic::{http::ReqwestExt, QueryBuilder};

pub async fn schema(
    account: &str,
    project: &str,
    branch: &str,
    subgraph_name: Option<&str>,
) -> Result<Option<String>, ApiError> {
    if let Some(subgraph_name) = subgraph_name {
        subgraph_schema(account, project, branch, subgraph_name).await.map(Some)
    } else {
        federated_graph_schema(account, project, branch).await
    }
}

async fn subgraph_schema(account: &str, project: &str, branch: &str, subgraph_name: &str) -> Result<String, ApiError> {
    let client = create_client().await?;
    let operation = FetchSubgraphSchemaQuery::build(FetchSubgraphSchemaArguments {
        account,
        project,
        subgraph_name,
        branch,
    });
    let response = client.post(API_URL).run_graphql(operation).await?;

    response
        .data
        .as_ref()
        .and_then(|data| data.subgraph.as_ref())
        .map(|subgraph| subgraph.schema.clone())
        .ok_or_else(|| ApiError::SubgraphsError(format!("{response:#?}")))
}

async fn federated_graph_schema(account: &str, project: &str, branch: &str) -> Result<Option<String>, ApiError> {
    let client = create_client().await?;
    let operation = FetchFederatedGraphSchemaQuery::build(FetchFederatedGraphSchemaArguments {
        account,
        project,
        branch,
    });
    let response = client.post(API_URL).run_graphql(operation).await?;

    response
        .data
        .as_ref()
        .and_then(|data| data.branch.as_ref())
        .ok_or_else(|| ApiError::SubgraphsError(format!("{response:#?}")))
        .map(|branch| branch.schema.clone())
}
