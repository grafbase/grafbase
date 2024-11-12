use super::super::schema;

#[derive(Clone, Default, cynic::QueryVariables)]
pub struct GraphSlugByIdArguments<'a> {
    pub id: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Account {
    pub slug: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Graph")]
pub struct GraphSlugByIdGraph {
    pub account: Account,
    pub slug: String,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum Node {
    Graph(GraphSlugByIdGraph),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GraphSlugByIdArguments")]
pub struct GraphSlugById {
    #[arguments(id: $id)]
    pub node: Option<Node>,
}
