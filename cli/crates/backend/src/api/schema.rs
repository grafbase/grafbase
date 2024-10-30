use crate::api::graphql::queries::{
    fetch_federated_graph_schema::{
        FetchFederatedGraphSchemaArguments, FetchFederatedGraphSchemaProductionBranchArguments,
        FetchFederatedGraphSchemaProductionBranchQuery, FetchFederatedGraphSchemaQuery,
    },
    fetch_subgraph_schema::{FetchSubgraphSchemaArguments, FetchSubgraphSchemaQuery},
};

use super::{client::create_client, errors::ApiError};
use common::environment::PlatformData;
use cynic::{http::ReqwestExt, QueryBuilder};

pub async fn schema(
    account: &str,
    project: &str,
    branch: Option<&str>,
    subgraph_name: Option<&str>,
) -> Result<Option<String>, ApiError> {
    if let Some(subgraph_name) = subgraph_name {
        subgraph_schema(account, project, branch, subgraph_name).await.map(Some)
    } else {
        federated_graph_schema(account, project, branch).await
    }
}

async fn subgraph_schema(
    account: &str,
    graph: &str,
    branch: Option<&str>,
    subgraph_name: &str,
) -> Result<String, ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client().await?;
    let operation = FetchSubgraphSchemaQuery::build(FetchSubgraphSchemaArguments {
        account,
        graph: Some(graph),
        subgraph_name,
        branch,
    });
    let response = client.post(&platform_data.api_url).run_graphql(operation).await?;

    response
        .data
        .as_ref()
        .and_then(|data| data.subgraph.as_ref())
        .map(|subgraph| subgraph.schema.clone())
        .ok_or_else(|| ApiError::SubgraphsError(format!("{response:#?}")))
}

async fn federated_graph_schema(account: &str, graph: &str, branch: Option<&str>) -> Result<Option<String>, ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client().await?;

    if let Some(branch) = branch {
        let operation =
            FetchFederatedGraphSchemaQuery::build(FetchFederatedGraphSchemaArguments { account, graph, branch });

        let response = client.post(&platform_data.api_url).run_graphql(operation).await?;

        response
            .data
            .as_ref()
            .and_then(|data| data.branch.as_ref())
            .ok_or_else(|| ApiError::SubgraphsError(format!("{response:#?}")))
            .map(|branch| branch.federated_schema.clone())
    } else {
        let operation =
            FetchFederatedGraphSchemaProductionBranchQuery::build(FetchFederatedGraphSchemaProductionBranchArguments {
                account,
                graph,
            });

        let response = client.post(&platform_data.api_url).run_graphql(operation).await?;

        response
            .data
            .as_ref()
            .and_then(|data| data.graph_by_account_slug.as_ref())
            .map(|graph| graph.production_branch.federated_schema.clone())
            .ok_or_else(|| ApiError::SubgraphsError(format!("{response:#?}")))
    }
}
