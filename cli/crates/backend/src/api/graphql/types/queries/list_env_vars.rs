use chrono::{DateTime, Utc};

use super::super::schema;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ListEnvironmentVariablesBySlugsArguments")]
pub struct ListEnvironmentVariablesBySlugs {
    #[arguments(accountSlug: $account_slug, graphSlug: $graph_slug)]
    pub graph_by_account_slug: Option<Graph>,
}

#[derive(cynic::QueryVariables)]
pub struct ListEnvironmentVariablesBySlugsArguments<'a> {
    pub account_slug: &'a str,
    pub graph_slug: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ListEnvironmentVariablesArguments")]
pub struct ListEnvironmentVariables {
    #[arguments(id: $graph_id)]
    pub node: Option<Node>,
}

#[derive(cynic::QueryVariables)]
pub struct ListEnvironmentVariablesArguments {
    pub graph_id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Graph {
    pub environment_variables: EnvironmentVariableConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct EnvironmentVariableConnection {
    pub edges: Vec<EnvironmentVariableEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct EnvironmentVariableEdge {
    pub node: EnvironmentVariable,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct EnvironmentVariable {
    pub name: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
    pub environments: Vec<BranchEnvironment>,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum Node {
    Graph(Graph),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum BranchEnvironment {
    Preview,
    Production,
}
