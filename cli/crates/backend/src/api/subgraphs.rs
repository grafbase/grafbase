use super::{
    client::create_client,
    consts::API_URL,
    errors::ApiError,
    graphql::queries::list_subgraphs::{ListSubgraphsArguments, ListSubgraphsQuery, Subgraph},
};
use cynic::{http::ReqwestExt, QueryBuilder};

/// The `grafbase subgraphs` command.
pub async fn subgraphs(account: &str, project: &str, branch: Option<&str>) -> Result<Vec<Subgraph>, ApiError> {
    let client = create_client().await?;
    let Some(branch) = branch else {
        return Err(ApiError::SubgraphsError(
            "A branch must be specified in order to list subgraphs".to_owned(),
        ));
    };

    let operation = ListSubgraphsQuery::build(ListSubgraphsArguments {
        account,
        project,
        branch,
    });

    let response = client.post(API_URL).run_graphql(operation).await?;
    let subgraphs = response
        .data
        .as_ref()
        .and_then(|branch| branch.branch.as_ref())
        .and_then(|branch| branch.subgraphs.as_deref());

    if let Some(subgraphs) = subgraphs {
        Ok(subgraphs.to_owned())
    } else {
        Err(ApiError::SubgraphsError(format!(
            "no subgraphs in response:\n{response:#?}",
        )))
    }
}
