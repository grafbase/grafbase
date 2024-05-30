use core::fmt;

use chrono::{DateTime, Utc};

use super::super::schema;

#[derive(cynic::QueryFragment)]
#[cynic(graphql_type = "Query", variables = "ListBranchesArguments")]
pub struct ListBranches {
    #[arguments(id: $graph_id)]
    pub node: Option<Node>,
}

#[derive(cynic::QueryVariables)]
pub struct ListBranchesArguments {
    pub graph_id: cynic::Id,
}

#[derive(cynic::QueryFragment)]
pub struct Account {
    pub slug: String,
}

#[derive(cynic::QueryFragment)]
pub struct Graph {
    pub branches: BranchConnection,
    pub account: Account,
    pub slug: String,
    pub production_branch: Branch,
}

#[derive(cynic::QueryFragment)]
pub struct BranchConnection {
    pub edges: Vec<BranchEdge>,
}

#[derive(cynic::QueryFragment)]
pub struct BranchEdge {
    pub node: Branch,
}

#[derive(cynic::QueryFragment)]
pub struct Branch {
    pub name: String,
    pub latest_deployment: Option<Deployment>,
}

#[derive(cynic::QueryFragment)]
pub struct Deployment {
    pub created_at: DateTime<Utc>,
    pub status: DeploymentStatus,
}

#[derive(cynic::Enum, Clone, Copy)]
pub enum DeploymentStatus {
    Queued,
    InProgress,
    Succeeded,
    Failed,
}

impl DeploymentStatus {
    pub fn failed(self) -> bool {
        matches!(self, Self::Failed)
    }
}

impl fmt::Display for DeploymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeploymentStatus::Queued => f.write_str("queued"),
            DeploymentStatus::InProgress => f.write_str("in progress"),
            DeploymentStatus::Succeeded => f.write_str("succeeded"),
            DeploymentStatus::Failed => f.write_str("failed"),
        }
    }
}

#[derive(cynic::InlineFragments)]
pub enum Node {
    Graph(Graph),
    #[cynic(fallback)]
    Unknown,
}
