use super::{FederatedGraphCompositionError, GraphDoesNotExistError};
use crate::api::graphql::types::schema;

#[derive(cynic::QueryVariables, Debug)]
pub struct DeleteSubgraphArguments<'a> {
    pub input: DeleteSubgraphInput<'a>,
}

#[derive(cynic::InputObject, Debug)]
pub struct DeleteSubgraphInput<'a> {
    pub account_slug: &'a str,
    pub graph_slug: Option<&'a str>,
    pub branch: &'a str,
    pub subgraph: &'a str,
    pub message: Option<&'a str>,
    pub dry_run: bool,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "DeleteSubgraphArguments")]
pub struct DeleteSubgraphMutation {
    #[arguments(input: $input)]
    pub delete_subgraph: DeleteSubgraphPayload,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum DeleteSubgraphPayload {
    DeleteSubgraphSuccess(#[allow(unused)] DeleteSubgraphSuccess),
    SubgraphNotFoundError(#[allow(unused)] SubgraphNotFoundError),
    ProjectDoesNotExistError(#[allow(unused)] ProjectDoesNotExistError),
    ProjectNotFederatedError(#[allow(unused)] ProjectNotFederatedError),
    ProjectBranchDoesNotExistError(#[allow(unused)] ProjectBranchDoesNotExistError),
    GraphDoesNotExistError(#[allow(unused)] GraphDoesNotExistError),
    GraphNotFederatedError(#[allow(unused)] GraphNotFederatedError),
    GraphBranchDoesNotExistError(#[allow(unused)] GraphBranchDoesNotExistError),
    FederatedGraphCompositionError(FederatedGraphCompositionError),
    DeleteSubgraphDeploymentFailure(DeleteSubgraphDeploymentFailure),
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DeleteSubgraphSuccess {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SubgraphNotFoundError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProjectDoesNotExistError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProjectNotFederatedError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProjectBranchDoesNotExistError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct GraphNotFederatedError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct GraphBranchDoesNotExistError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DeleteSubgraphDeploymentFailure {
    pub deployment_error: String,
}
