pub(crate) mod submit_trusted_documents;

use super::schema;

#[derive(cynic::InputObject, Clone, Debug)]
pub struct GraphCreateInput<'a> {
    pub account_id: cynic::Id,
    pub graph_slug: &'a str,
}

#[derive(cynic::QueryVariables)]
pub struct GraphCreateArguments<'a> {
    pub input: GraphCreateInput<'a>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SlugTooLongError {
    pub __typename: String,
    pub max_length: i32,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SlugInvalidError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SlugAlreadyExistsError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct GraphCreateSuccess {
    pub __typename: String,
    pub graph: Graph,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Account {
    pub slug: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Graph {
    pub id: cynic::Id,
    pub slug: String,
    pub account: Account,
    pub production_branch: Branch,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "GraphCreateArguments")]
pub struct GraphCreate {
    #[arguments(input: $input)]
    pub graph_create: GraphCreatePayload,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct CurrentPlanLimitReachedError {
    pub __typename: String,
    pub max: i32,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Branch {
    pub domains: Vec<String>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct AccountDoesNotExistError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DisabledAccountError {
    pub __typename: String,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum GraphCreatePayload {
    GraphCreateSuccess(GraphCreateSuccess),
    SlugAlreadyExistsError(SlugAlreadyExistsError),
    DisabledAccountError(DisabledAccountError),
    SlugInvalidError(SlugInvalidError),
    SlugTooLongError(SlugTooLongError),
    AccountDoesNotExistError(AccountDoesNotExistError),
    CurrentPlanLimitReachedError(CurrentPlanLimitReachedError),
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(cynic::QueryFragment, Debug)]
pub struct GraphDoesNotExistError {
    pub __typename: String,
}

#[derive(cynic::QueryVariables)]
pub struct BranchCreateArguments<'a> {
    pub input: BranchCreateInput<'a>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct BranchAlreadyExistsError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct GraphNotSelfHostedError {
    pub __typename: String,
}

#[derive(cynic::InputObject, Debug)]
pub struct BranchCreateInput<'a> {
    pub account_slug: &'a str,
    pub graph_slug: &'a str,
    pub branch_name: &'a str,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "BranchCreateArguments")]
pub struct BranchCreate {
    #[arguments(input: $input)]
    pub branch_create: BranchCreatePayload,
}

#[derive(cynic::InlineFragments)]
pub enum BranchCreatePayload {
    Success(BranchCreateSuccess),
    BranchAlreadyExists(BranchAlreadyExistsError),
    GraphDoesNotExist(GraphDoesNotExistError),
    GraphNotSelfHosted(GraphNotSelfHostedError),
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query")]
pub struct BranchCreateSuccess {
    pub __typename: String,
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Mutation", variables = "BranchDeleteArguments")]
pub struct BranchDelete {
    #[arguments(accountSlug: $account_slug, graphSlug: $graph_slug, branchName: $branch_name)]
    pub branch_delete: BranchDeletePayload,
}

#[derive(cynic::InlineFragments)]
pub enum BranchDeletePayload {
    Success(BranchDeleteSuccess),
    BranchDoesNotExist(BranchDoesNotExistError),
    CannotDeleteProductionBranch(CannotDeleteProductionBranchError),
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query")]
pub struct BranchDeleteSuccess {
    pub __typename: String,
}

#[derive(cynic::QueryFragment)]
pub struct BranchDoesNotExistError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment)]
pub struct CannotDeleteProductionBranchError {
    pub __typename: String,
}

#[derive(cynic::QueryVariables)]
pub struct BranchDeleteArguments<'a> {
    pub account_slug: &'a str,
    pub graph_slug: &'a str,
    pub branch_name: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PublishSuccess {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct FederatedGraphCompositionError {
    pub messages: Vec<String>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SchemaRegistryBranchDoesNotExistError {
    pub __typename: String,
}

#[derive(cynic::InlineFragments, Debug)]
#[allow(dead_code)]
pub(crate) enum PublishPayload {
    PublishSuccess(PublishSuccess),
    NoChange(PublishNoChange),
    GraphDoesNotExist(GraphDoesNotExistError),
    FederatedGraphCompositionError(FederatedGraphCompositionError),
    BranchDoesNotExistError(SchemaRegistryBranchDoesNotExistError),
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(cynic::QueryVariables, Debug)]
pub struct SubgraphCreateArguments<'a> {
    pub input: PublishInput<'a>,
}

#[derive(cynic::InputObject, Debug)]
pub struct PublishInput<'a> {
    pub branch: Option<&'a str>,
    pub message: Option<&'a str>,
    pub account_slug: &'a str,
    pub graph_slug: Option<&'a str>,
    pub schema: &'a str,
    pub subgraph: &'a str,
    pub url: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "SubgraphCreateArguments")]
pub(crate) struct SubgraphPublish {
    #[arguments(input: $input)]
    pub(crate) publish: PublishPayload,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SchemaCheck {
    pub id: cynic::Id,
    pub error_count: i32,
    pub validation_check_errors: Vec<ValidationCheckError>,
    pub composition_check_errors: Vec<CompositionCheckError>,
    pub operation_check_errors: Vec<OperationCheckError>,
    pub lint_check_errors: Vec<LintCheckError>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ValidationCheckError {
    pub message: String,
    pub title: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct CompositionCheckError {
    pub message: String,
    pub title: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct OperationCheckError {
    pub message: String,
    pub title: String,
    pub severity: SchemaCheckErrorSeverity,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct LintCheckError {
    pub message: String,
    pub title: String,
    pub severity: SchemaCheckErrorSeverity,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum SchemaCheckErrorSeverity {
    Error,
    Warning,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct SubgraphNameMissingOnFederatedGraphError {
    __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub(crate) struct PublishNoChange {
    __typename: String,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum SchemaCheckPayload {
    SchemaCheck(SchemaCheck),
    SubgraphNameMissingOnFederatedGraphError(SubgraphNameMissingOnFederatedGraphError),
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(cynic::InputObject, Debug)]
pub struct SchemaCheckCreateInput<'a> {
    pub account_slug: &'a str,
    pub graph_slug: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub subgraph_name: Option<&'a str>,
    pub schema: &'a str,
    pub git_commit: Option<SchemaCheckGitCommitInput>,
}

#[derive(cynic::InputObject, Debug)]
pub struct SchemaCheckGitCommitInput {
    pub branch: String,
    pub commit_sha: String,
    pub message: String,
    pub author_name: String,
}

#[derive(cynic::QueryVariables)]
pub struct SchemaCheckCreateArguments<'a> {
    pub input: SchemaCheckCreateInput<'a>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "SchemaCheckCreateArguments")]
pub struct SchemaCheckCreate {
    #[arguments(input: $input)]
    pub schema_check_create: Option<SchemaCheckPayload>,
}
