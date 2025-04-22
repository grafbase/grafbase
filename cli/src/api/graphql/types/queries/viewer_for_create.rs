use super::super::schema;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub(crate) struct Viewer {
    pub viewer: Option<User>,
}

#[derive(cynic::QueryFragment, Debug)]
pub(crate) struct User {
    #[arguments(last: 100)]
    pub organizations: OrganizationConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub(crate) struct OrganizationConnection {
    pub nodes: Vec<Organization>,
}

#[derive(cynic::QueryFragment, Debug)]
pub(crate) struct Organization {
    pub id: cynic::Id,
    pub name: String,
    pub slug: String,
}
