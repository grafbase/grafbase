use super::super::schema;

#[derive(cynic::QueryVariables)]
pub struct ListSubgraphsArguments<'a> {
    pub account: &'a str,
    pub graph: &'a str,
    pub branch: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ListSubgraphsArguments")]
pub struct ListSubgraphsQuery {
    #[arguments(accountSlug: $account, graphSlug: $graph, name: $branch)]
    pub branch: Option<Branch>,
}

#[derive(cynic::QueryVariables)]
pub struct ListSubgraphsForProductionBranchArguments<'a> {
    pub account: &'a str,
    pub graph: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ListSubgraphsForProductionBranchArguments")]
pub struct ListSubgraphsForProductionBranchQuery {
    #[arguments(accountSlug: $account, graphSlug: $graph)]
    pub graph_by_account_slug: Option<Graph>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Graph {
    pub production_branch: Branch,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Branch {
    pub name: String,
    pub subgraphs: Option<Vec<Subgraph>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Subgraph {
    pub name: String,
}
