use crate::api::graphql::types::schema;

#[derive(cynic::QueryVariables)]
pub struct FetchOrganizationBySlugArguments<'a> {
    pub slug: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "FetchOrganizationBySlugArguments")]
pub struct FetchOrganizationBySlug {
    #[arguments(slug: $slug)]
    pub account_by_slug: Option<Account>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Account {
    pub id: cynic::Id,
}
