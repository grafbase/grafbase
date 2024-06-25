use super::{super::schema, environment_variable_upsert_by_slugs::BranchEnvironment};

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Mutation",
    variables = "EnvironmentVariableDeleteWithValuesBySlugVariables"
)]
pub struct EnvironmentVariableDeleteWithValuesBySlug {
    #[arguments(input: { accountSlug: $account_slug, projectSlug: $project_slug, environments: $environments, name: $name })]
    pub environment_variable_delete_with_values_by_slug: EnvironmentVariableDeleteByValuesPayload,
}

#[derive(cynic::QueryVariables)]
pub struct EnvironmentVariableDeleteWithValuesBySlugVariables<'a> {
    pub account_slug: &'a str,
    pub project_slug: &'a str,
    pub environments: Vec<BranchEnvironment>,
    pub name: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Mutation",
    variables = "EnvironmentVariableDeleteWithValuesVariables"
)]
pub struct EnvironmentVariableDeleteWithValues {
    #[arguments(input: { projectId: $project_id, environments: $environments, name: $name })]
    pub environment_variable_delete_with_values: EnvironmentVariableDeleteByValuesPayload,
}

#[derive(cynic::QueryVariables)]
pub struct EnvironmentVariableDeleteWithValuesVariables<'a> {
    pub project_id: cynic::Id,
    pub environments: Vec<BranchEnvironment>,
    pub name: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct EnvironmentVariableDoesNotExistError {
    pub __typename: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct EnvironmentVariableDeleteByValuesSuccess {
    pub __typename: String,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum EnvironmentVariableDeleteByValuesPayload {
    #[allow(dead_code)]
    EnvironmentVariableDeleteByValuesSuccess(EnvironmentVariableDeleteByValuesSuccess),
    #[allow(dead_code)]
    EnvironmentVariableDoesNotExistError(EnvironmentVariableDoesNotExistError),
    #[cynic(fallback)]
    Unknown(String),
}
