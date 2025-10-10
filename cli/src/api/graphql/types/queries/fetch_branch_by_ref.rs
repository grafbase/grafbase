use super::super::schema;

#[derive(cynic::QueryVariables)]
pub struct FetchBranchByRefArguments<'a> {
    pub account_slug: &'a str,
    pub graph_slug: &'a str,
    pub branch_name: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "FetchBranchByRefArguments")]
pub struct FetchBranchByRefQuery {
    #[arguments(accountSlug: $account_slug, graphSlug: $graph_slug, name: $branch_name)]
    pub branch: Option<Branch>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Branch {
    pub id: cynic::Id,
}
