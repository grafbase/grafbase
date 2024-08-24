use chrono::{DateTime, Utc};
use cynic::{http::ReqwestExt, MutationBuilder, QueryBuilder};

use crate::api::graphql::mutations::{BranchCreate, BranchCreateArguments, BranchCreateInput};

use super::{
    client::create_client,
    consts::api_url,
    errors::{ApiError, BranchError},
    graphql::{
        mutations::{BranchDelete, BranchDeleteArguments, BranchDeletePayload},
        queries::list_branches::{ListBranches, ListBranchesArguments, Node},
    },
};

pub struct Branch {
    pub account: String,
    pub graph: String,
    pub branch: String,
    pub is_production: bool,
    pub last_updated: Option<DateTime<Utc>>,
    pub status: Option<String>,
}

/// # Errors
///
/// See [`ApiError`]
pub async fn delete(account_slug: &str, project_slug: &str, branch_name: &str) -> Result<(), ApiError> {
    let client = create_client().await?;

    let operation = BranchDelete::build(BranchDeleteArguments {
        account_slug,
        project_slug,
        branch_name,
    });

    let cynic::GraphQlResponse { data, errors } = client.post(api_url()).run_graphql(operation).await?;

    if let Some(data) = data {
        match data.branch_delete {
            BranchDeletePayload::Success(_) => Ok(()),
            BranchDeletePayload::BranchDoesNotExist(_) => {
                Err(BranchError::BranchDoesNotExist(format!("{account_slug}/{project_slug}@{branch_name}")).into())
            }
            BranchDeletePayload::CannotDeleteProductionBranch(_) => Err(
                BranchError::CannotDeleteProductionBranchError(format!("{account_slug}/{project_slug}@{branch_name}"))
                    .into(),
            ),
            BranchDeletePayload::Unknown(error) => Err(BranchError::Unknown(error).into()),
        }
    } else {
        Err(ApiError::RequestError(format!("{errors:#?}")))
    }
}

pub async fn list() -> Result<Vec<Branch>, ApiError> {
    let project_metadata = crate::api::project_metadata()?;

    let operation = ListBranches::build(ListBranchesArguments {
        graph_id: project_metadata.graph_id().into(),
    });

    let client = create_client().await?;
    let cynic::GraphQlResponse { data, errors } = client.post(api_url()).run_graphql(operation).await?;

    match (data.and_then(|d| d.node), errors) {
        (Some(Node::Graph(graph)), _) => {
            let branches = graph
                .branches
                .edges
                .into_iter()
                .map(|edge| {
                    let is_production = edge.node.name == graph.production_branch.name;

                    let (last_updated, status) = match edge.node.latest_deployment {
                        Some(deployment) => (Some(deployment.created_at), Some(deployment.status.to_string())),
                        None => (None, None),
                    };

                    Branch {
                        account: graph.account.slug.clone(),
                        graph: graph.slug.clone(),
                        branch: edge.node.name,
                        is_production,
                        last_updated,
                        status,
                    }
                })
                .collect();

            Ok(branches)
        }
        (_, errors) => Err(ApiError::RequestError(format!("{errors:#?}"))),
    }
}

pub async fn create(account_slug: &str, graph_slug: &str, branch_name: &str) -> Result<(), ApiError> {
    let client = create_client().await?;

    let operation = BranchCreate::build(BranchCreateArguments {
        input: BranchCreateInput {
            account_slug,
            graph_slug,
            branch_name,
        },
    });

    let cynic::GraphQlResponse { data, errors } = client.post(api_url()).run_graphql(operation).await?;

    let Some(data) = data else {
        return Err(ApiError::RequestError(format!("{errors:#?}")));
    };

    match data.branch_create {
        super::graphql::mutations::BranchCreatePayload::Success(_) => Ok(()),
        super::graphql::mutations::BranchCreatePayload::BranchAlreadyExists(_) => Err(ApiError::BranchError(
            BranchError::BranchAlreadyExists(format!("{account_slug}/{graph_slug}@{branch_name}")),
        )),
        super::graphql::mutations::BranchCreatePayload::GraphDoesNotExist(_) => Err(ApiError::GraphDoesNotExist),
        super::graphql::mutations::BranchCreatePayload::GraphNotSelfHosted(_) => Err(ApiError::GraphNotSelfHosted),
        super::graphql::mutations::BranchCreatePayload::Unknown(error) => Err(BranchError::Unknown(error).into()),
    }
}
