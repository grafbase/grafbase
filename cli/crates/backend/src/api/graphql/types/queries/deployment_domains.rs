use super::super::schema;

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "DeploymentDomainsArguments")]
pub struct DeploymentDomains {
    #[arguments(id: $deployment_id)]
    pub node: Option<Node>,
}

#[derive(cynic::InlineFragments)]
pub enum Node {
    Deployment(Deployment),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::QueryVariables)]
pub struct DeploymentDomainsArguments {
    pub deployment_id: cynic::Id,
}

#[derive(cynic::QueryFragment)]
pub struct Deployment {
    pub branch: Branch,
}

#[derive(cynic::QueryFragment)]
pub struct Branch {
    pub domains: Vec<String>,
}
