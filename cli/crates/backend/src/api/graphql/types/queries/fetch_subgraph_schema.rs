use super::super::schema;

#[derive(cynic::QueryVariables)]
pub struct FetchSubgraphSchemaArguments<'a> {
    pub account: &'a str,
    pub project: &'a str,
    pub subgraph_name: &'a str,
    pub branch: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "FetchSubgraphSchemaArguments")]
pub struct FetchSubgraphSchemaQuery {
    #[arguments(accountSlug: $account, projectSlug: $project, branch: $branch, subgraphName: $subgraph_name)]
    pub subgraph: Option<Subgraph>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Subgraph {
    pub schema: String,
}
