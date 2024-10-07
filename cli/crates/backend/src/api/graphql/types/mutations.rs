pub(crate) mod submit_trusted_documents;

use super::schema;

#[derive(cynic::InputObject, Clone, Debug)]
pub struct GraphCreateInput<'a> {
    pub account_id: cynic::Id,
    pub graph_slug: &'a str,
    pub repo_root_path: Option<&'a str>,
    pub environment_variables: Vec<EnvironmentVariableSpecification<'a>>,
    pub graph_mode: GraphMode,
}

#[derive(cynic::Enum, Clone, Debug, Copy)]
pub enum GraphMode {
    Managed,
    SelfHosted,
}

#[derive(cynic::InputObject, Clone, Debug)]
pub struct EnvironmentVariableSpecification<'a> {
    pub name: &'a str,
    pub value: &'a str,
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
    #[arguments(last: 5)]
    pub api_keys: GraphApiKeyConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct GraphApiKeyConnection {
    pub nodes: Vec<GraphApiKey>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct GraphApiKey {
    pub key: String,
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "GraphCreateArguments")]
pub struct GraphCreate {
    #[arguments(input: $input)]
    pub graph_create: GraphCreatePayload,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct InvalidDatabaseRegionsError {
    pub __typename: String,
    pub invalid: Vec<String>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct EmptyDatabaseRegionsError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DuplicateDatabaseRegionsError {
    pub __typename: String,
    pub duplicates: Vec<String>,
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

#[derive(cynic::QueryFragment, Debug)]
pub struct EnvironmentVariableCountLimitExceededError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct InvalidEnvironmentVariablesError {
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
    InvalidEnvironmentVariablesError(InvalidEnvironmentVariablesError),
    EnvironmentVariableCountLimitExceededError(EnvironmentVariableCountLimitExceededError),
    InvalidDatabaseRegionsError(InvalidDatabaseRegionsError),
    DuplicateDatabaseRegionsError(DuplicateDatabaseRegionsError),
    EmptyDatabaseRegionsError(EmptyDatabaseRegionsError),
    #[cynic(fallback)]
    Unknown(String),
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProjectDoesNotExistError {
    pub __typename: String,
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
    #[arguments(accountSlug: $account_slug, projectSlug: $project_slug, branchName: $branch_name)]
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
    pub project_slug: &'a str,
    pub branch_name: &'a str,
}

#[derive(cynic::InputObject, Clone, Debug)]
#[cynic(rename_all = "camelCase")]
pub struct DeploymentCreateInput<'a> {
    pub archive_file_size: i32,
    pub branch: Option<&'a str>,
    pub graph_id: cynic::Id,
}

#[derive(cynic::QueryVariables)]
pub struct DeploymentCreateArguments<'a> {
    pub input: DeploymentCreateInput<'a>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "DeploymentCreateArguments")]
pub struct DeploymentCreate {
    #[arguments(input: $input)]
    pub deployment_create: DeploymentCreatePayload,
}

#[derive(cynic::InputObject, Clone, Debug)]
#[cynic(rename_all = "camelCase")]
pub struct DeploymentBySlugCreateInput<'a> {
    pub archive_file_size: i32,
    pub branch: Option<&'a str>,
    pub graph_slug: Option<&'a str>,
    pub account_slug: &'a str,
}

#[derive(cynic::QueryVariables)]
pub struct DeploymentCreateBySlugArguments<'a> {
    pub input: DeploymentBySlugCreateInput<'a>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "DeploymentCreateBySlugArguments")]
pub struct DeploymentCreatebySlug {
    #[arguments(input: $input)]
    pub deployment_create_by_slug: DeploymentCreatePayload,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DeploymentCreateSuccess {
    pub __typename: String,
    pub presigned_url: String,
    pub deployment: Deployment,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Deployment {
    pub id: cynic::Id,
    pub graph: Graph,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DailyDeploymentCountLimitExceededError {
    pub __typename: String,
    pub limit: i32,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ArchiveFileSizeLimitExceededError {
    pub __typename: String,
    pub limit: i32,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum DeploymentCreatePayload {
    DeploymentCreateSuccess(DeploymentCreateSuccess),
    ProjectDoesNotExistError(ProjectDoesNotExistError),
    ArchiveFileSizeLimitExceededError(ArchiveFileSizeLimitExceededError),
    DailyDeploymentCountLimitExceededError(DailyDeploymentCountLimitExceededError),
    #[cynic(fallback)]
    Unknown(String),
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
pub enum PublishPayload {
    PublishSuccess(PublishSuccess),
    ProjectDoesNotExistError(ProjectDoesNotExistError),
    FederatedGraphCompositionError(FederatedGraphCompositionError),
    BranchDoesNotExistError(SchemaRegistryBranchDoesNotExistError),
    #[cynic(fallback)]
    Unknown(String),
}

cynic::impl_scalar!(url::Url, schema::Url);

#[derive(cynic::QueryVariables, Debug)]
pub struct SubgraphCreateArguments<'a> {
    pub input: PublishInput<'a>,
}

#[derive(cynic::InputObject, Debug)]
pub struct PublishInput<'a> {
    pub account_slug: &'a str,
    pub project_slug: &'a str,
    pub branch: Option<&'a str>,
    pub subgraph: &'a str,
    pub url: &'a str,
    pub schema: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "SubgraphCreateArguments")]
pub struct SubgraphPublish {
    #[arguments(input: $input)]
    pub publish: PublishPayload,
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
pub struct SubgraphNameMissingOnFederatedProjectError {
    __typename: String,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum SchemaCheckPayload {
    SchemaCheck(SchemaCheck),
    SubgraphNameMissingOnFederatedProjectError(SubgraphNameMissingOnFederatedProjectError),
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
