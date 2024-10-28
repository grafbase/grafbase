use super::{
    client::create_client,
    errors::ApiError,
    graphql::queries::list_subgraphs::{
        ListSubgraphsArguments, ListSubgraphsForProductionBranchArguments, ListSubgraphsForProductionBranchQuery,
        ListSubgraphsQuery, Subgraph,
    },
};
use common::environment::PlatformData;
use cynic::{http::ReqwestExt, QueryBuilder};

/// The `grafbase subgraphs` command. Returns (branch name, subgraphs).
pub async fn subgraphs(
    account: &str,
    project: &str,
    branch: Option<&str>,
) -> Result<(String, Vec<Subgraph>), ApiError> {
    match branch {
        Some(branch) => subgraphs_with_branch(account, project, branch).await,
        None => subgraphs_production_branch(account, project).await,
    }
}

async fn subgraphs_with_branch(account: &str, graph: &str, branch: &str) -> Result<(String, Vec<Subgraph>), ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client().await?;

    let operation = ListSubgraphsQuery::build(ListSubgraphsArguments { account, graph, branch });

    let response = client.post(&platform_data.api_url).run_graphql(operation).await?;
    let subgraphs = response
        .data
        .as_ref()
        .and_then(|branch| branch.branch.as_ref())
        .and_then(|branch| Some(&branch.name).zip(branch.subgraphs.as_deref()));

    if let Some((branch, subgraphs)) = subgraphs {
        Ok((branch.to_owned(), subgraphs.to_owned()))
    } else {
        Err(ApiError::SubgraphsError(format!(
            "no subgraphs in response:\n{response:#?}",
        )))
    }
}

async fn subgraphs_production_branch(account: &str, graph: &str) -> Result<(String, Vec<Subgraph>), ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client().await?;

    let operation =
        ListSubgraphsForProductionBranchQuery::build(ListSubgraphsForProductionBranchArguments { account, graph });

    let response = client.post(&platform_data.api_url).run_graphql(operation).await?;
    let subgraphs = response
        .data
        .as_ref()
        .and_then(|query| query.graph_by_account_slug.as_ref())
        .map(|project| &project.production_branch)
        .and_then(|branch| Some(&branch.name).zip(branch.subgraphs.as_deref()));

    if let Some((branch, subgraphs)) = subgraphs {
        Ok((branch.to_owned(), subgraphs.to_owned()))
    } else {
        Err(ApiError::SubgraphsError(format!(
            "no subgraphs in response:\n{response:#?}",
        )))
    }
}
