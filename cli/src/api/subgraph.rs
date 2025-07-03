use super::{
    client::create_client,
    errors::ApiError,
    graphql::{
        mutations::delete_subgraph::{
            DeleteSubgraphArguments, DeleteSubgraphInput, DeleteSubgraphMutation, DeleteSubgraphPayload,
        },
        queries::list_subgraphs::{
            ListSubgraphsArguments, ListSubgraphsForProductionBranchArguments, ListSubgraphsForProductionBranchQuery,
            ListSubgraphsQuery, Subgraph,
        },
    },
};
use crate::common::environment::PlatformData;
use cynic::{MutationBuilder, QueryBuilder, http::ReqwestExt};

/// List all subgraphs
pub async fn list(account: &str, graph: &str, branch: Option<&str>) -> Result<(String, Vec<Subgraph>), ApiError> {
    match branch {
        Some(branch) => subgraphs_with_branch(account, graph, branch).await,
        None => subgraphs_production_branch(account, graph).await,
    }
}

/// Delete a subgraph
pub async fn delete(account: &str, graph: &str, branch: &str, name: &str) -> Result<(), ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client()?;

    let operation = DeleteSubgraphMutation::build(DeleteSubgraphArguments {
        input: DeleteSubgraphInput {
            account_slug: account,
            graph_slug: Some(graph),
            project_slug: None,
            branch,
            subgraph: name,
            message: None,
            dry_run: false,
        },
    });

    let response = client.post(platform_data.api_url()).run_graphql(operation).await?;

    if let Some(errors) = response.errors {
        return Err(ApiError::SubgraphsError(format!(
            "failed to delete subgraph: {:?}",
            errors
        )));
    }

    match response.data.map(|data| data.delete_subgraph) {
        Some(DeleteSubgraphPayload::DeleteSubgraphSuccess(_)) => Ok(()),
        Some(DeleteSubgraphPayload::SubgraphNotFoundError(_)) => {
            Err(ApiError::SubgraphsError("Subgraph not found".to_string()))
        }
        Some(DeleteSubgraphPayload::GraphDoesNotExistError(_)) => {
            Err(ApiError::SubgraphsError("Graph does not exist".to_string()))
        }
        Some(DeleteSubgraphPayload::GraphNotFederatedError(_)) => {
            Err(ApiError::SubgraphsError("Graph is not federated".to_string()))
        }
        Some(DeleteSubgraphPayload::GraphBranchDoesNotExistError(_)) => {
            Err(ApiError::SubgraphsError("Graph branch does not exist".to_string()))
        }
        Some(DeleteSubgraphPayload::ProjectDoesNotExistError(_)) => {
            Err(ApiError::SubgraphsError("Project does not exist".to_string()))
        }
        Some(DeleteSubgraphPayload::ProjectNotFederatedError(_)) => {
            Err(ApiError::SubgraphsError("Project is not federated".to_string()))
        }
        Some(DeleteSubgraphPayload::ProjectBranchDoesNotExistError(_)) => {
            Err(ApiError::SubgraphsError("Project branch does not exist".to_string()))
        }
        Some(DeleteSubgraphPayload::FederatedGraphCompositionError(err)) => Err(ApiError::SubgraphsError(format!(
            "Federation composition error: {:?}",
            err.messages
        ))),
        Some(DeleteSubgraphPayload::DeleteSubgraphDeploymentFailure(err)) => Err(ApiError::SubgraphsError(format!(
            "Deployment failure: {}",
            err.deployment_error
        ))),
        Some(DeleteSubgraphPayload::Unknown(typename)) => {
            Err(ApiError::SubgraphsError(format!("Unknown error: {}", typename)))
        }
        None => Err(ApiError::SubgraphsError(
            "No data in delete subgraph response".to_string(),
        )),
    }
}

async fn subgraphs_with_branch(account: &str, graph: &str, branch: &str) -> Result<(String, Vec<Subgraph>), ApiError> {
    let platform_data = PlatformData::get();
    let client = create_client()?;

    let operation = ListSubgraphsQuery::build(ListSubgraphsArguments { account, graph, branch });

    let response = client.post(platform_data.api_url()).run_graphql(operation).await?;
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
    let client = create_client()?;

    let operation =
        ListSubgraphsForProductionBranchQuery::build(ListSubgraphsForProductionBranchArguments { account, graph });

    let response = client.post(platform_data.api_url()).run_graphql(operation).await?;
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
