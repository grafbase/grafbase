use super::super::schema;

#[derive(cynic::QueryVariables)]
pub struct FetchFederatedGraphSchemaArguments<'a> {
    pub account: &'a str,
    pub graph: &'a str,
    pub branch: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "FetchFederatedGraphSchemaArguments")]
pub struct FetchFederatedGraphSchemaQuery {
    #[arguments(accountSlug: $account, graphSlug: $graph, name: $branch)]
    pub branch: Option<Branch>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct BranchConnection {
    pub nodes: Vec<Branch>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Branch {
    pub name: String,
    pub federated_schema: Option<String>,
}

#[derive(cynic::QueryVariables)]
pub struct FetchFederatedGraphSchemaProductionBranchArguments<'a> {
    pub account: &'a str,
    pub graph: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Query",
    variables = "FetchFederatedGraphSchemaProductionBranchArguments"
)]
pub struct FetchFederatedGraphSchemaProductionBranchQuery {
    #[arguments(accountSlug: $account, graphSlug: $graph)]
    pub graph_by_account_slug: Option<Graph>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Graph {
    pub production_branch: Branch,
}
