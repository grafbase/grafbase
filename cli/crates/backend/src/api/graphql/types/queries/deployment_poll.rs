use chrono::{DateTime, Utc};

use super::{super::schema, list_branches::DeploymentStatus};

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "DeploymentLogsArguments")]
pub struct DeploymentLogs {
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
pub struct DeploymentLogsArguments {
    pub deployment_id: cynic::Id,
}

#[derive(cynic::QueryFragment)]
pub struct Deployment {
    pub finished_at: Option<DateTime<Utc>>,
    pub log_entries: Vec<DeploymentLogEntry>,
    pub status: DeploymentStatus,
}

#[derive(cynic::QueryFragment)]
pub struct DeploymentLogEntry {
    pub created_at: DateTime<Utc>,
    pub message: String,
}
