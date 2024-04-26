use super::super::schema;

#[derive(cynic::QueryFragment, Debug)]
pub struct ValueTooLongError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ReservedPrefixError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProjectDoesNotExistError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct NameTooLongError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct NameContainsInvalidCharactersError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "EnvironmentVariableUpsertBySlugsVariables")]
pub struct EnvironmentVariableUpsertBySlugs {
    #[arguments(input: { accountSlug: $account_slug, projectSlug: $project_slug, environments: $environments, name: $name, value: $value })]
    pub environent_variable_upsert_by_slug: EnvironmentVariableCreatePayload,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "EnvironmentVariableUpsertVariables")]
pub struct EnvironmentVariableUpsert {
    #[arguments(input: { projectId: $project_id, environments: $environments, name: $name, value: $value })]
    pub environent_variable_upsert: EnvironmentVariableCreatePayload,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum BranchEnvironment {
    Preview,
    Production,
}

#[derive(cynic::QueryVariables)]
pub struct EnvironmentVariableUpsertBySlugsVariables<'a> {
    pub account_slug: &'a str,
    pub project_slug: &'a str,
    pub environments: Vec<BranchEnvironment>,
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(cynic::QueryVariables)]
pub struct EnvironmentVariableUpsertVariables<'a> {
    pub project_id: cynic::Id,
    pub environments: Vec<BranchEnvironment>,
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct EnvironmentVariableCreateSuccess {
    pub __typename: String,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum EnvironmentVariableCreatePayload {
    EnvironmentVariableCreateSuccess(EnvironmentVariableCreateSuccess),
    NameTooLongError(NameTooLongError),
    NameContainsInvalidCharactersError(NameContainsInvalidCharactersError),
    ValueTooLongError(ValueTooLongError),
    ReservedPrefixError(ReservedPrefixError),
    ProjectDoesNotExistError(ProjectDoesNotExistError),
    #[cynic(fallback)]
    Unknown,
}
