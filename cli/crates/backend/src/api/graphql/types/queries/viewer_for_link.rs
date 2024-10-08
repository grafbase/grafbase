use super::super::schema;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct Viewer {
    pub viewer: Option<User>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct GraphConnection {
    pub nodes: Vec<Graph>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct OrganizationConnection {
    pub nodes: Vec<Organization>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Graph {
    pub id: cynic::Id,
    pub slug: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct User {
    pub personal_account: Option<PersonalAccount>,
    #[arguments(last: 100)]
    pub organizations: OrganizationConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PersonalAccount {
    pub id: cynic::Id,
    pub name: String,
    pub slug: String,
    #[arguments(last: 100)]
    pub graphs: GraphConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Member {
    pub account: Account,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Account {
    pub id: cynic::Id,
    pub name: String,
    pub slug: String,
    #[arguments(last: 100)]
    pub graphs: GraphConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Organization {
    pub id: cynic::Id,
    pub name: String,
    pub slug: String,
    #[arguments(last: 100)]
    pub graphs: GraphConnection,
}
