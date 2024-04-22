use super::super::schema;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "ListBranchesArguments")]
pub struct ListBranches {
    #[arguments(id: $project_id)]
    pub node: Option<Node>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct ListBranchesArguments {
    pub project_id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Project {
    pub branches: BranchConnection,
    pub account_slug: String,
    pub slug: String,
    pub production_branch: Branch,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct BranchConnection {
    pub edges: Vec<BranchEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct BranchEdge {
    pub node: Branch,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Branch {
    pub name: String,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum Node {
    Project(Project),
    #[cynic(fallback)]
    Unknown,
}
