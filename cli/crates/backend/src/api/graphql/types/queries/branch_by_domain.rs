use super::super::schema;

#[derive(cynic::QueryFragment, Debug)]
pub struct Graph {
    pub slug: String,
    pub account: Account,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Account {
    pub slug: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Branch {
    pub name: String,
    pub graph: Graph,
}

#[derive(cynic::QueryVariables)]
pub struct BranchByDomainArguments<'a> {
    pub domain: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "BranchByDomainArguments")]
pub struct BranchByDomain {
    #[arguments(domain: $domain)]
    pub branch_by_domain: Option<Branch>,
}
