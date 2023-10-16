use super::super::schema;

#[derive(cynic::QueryFragment, Debug)]
pub struct Project {
    pub slug: String,
    pub account_slug: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Branch {
    pub name: String,
    pub project: Project,
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
