use super::super::schema;

#[derive(cynic::QueryVariables, Debug)]
pub struct SubgraphSchemasByBranchVariables<'a> {
    pub account_slug: &'a str,
    pub graph_slug: &'a str,
    pub name: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "SubgraphSchemasByBranchVariables")]
pub struct SubgraphSchemasByBranch {
    #[arguments(accountSlug: $account_slug, graphSlug: $graph_slug, name: $name)]
    pub branch: Option<Branch>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Branch {
    pub subgraphs: Vec<Subgraph>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Subgraph {
    pub name: String,
    pub schema: String,
    pub url: String,
}
