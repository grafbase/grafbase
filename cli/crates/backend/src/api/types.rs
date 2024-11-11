use super::errors::LoginApiError;
use serde::{Deserialize, Serialize};

pub enum LoginMessage {
    CallbackUrl(String),
    Done,
    Error(LoginApiError),
}

#[derive(Debug)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub personal: bool,
}

#[derive(Debug)]
pub struct AccountWithGraphs {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub personal: bool,
    pub graphs: Vec<Graph>,
}

#[derive(Debug)]
pub struct Graph {
    pub id: String,
    pub slug: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    #[serde(alias = "project_id")]
    graph_id: String,
}

impl ProjectMetadata {
    pub fn new(graph_id: String) -> Self {
        Self { graph_id }
    }

    pub fn graph_id(&self) -> String {
        self.graph_id.replace("project", "graph")
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for ProjectMetadata {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("must parse")
    }
}
