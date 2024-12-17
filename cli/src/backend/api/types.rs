use super::errors::LoginApiError;
use serde::{Deserialize, Serialize};

pub enum LoginMessage {
    CallbackUrl(String),
    Done,
    Error(LoginApiError),
}

#[derive(Debug)]
pub(crate) struct Account {
    pub id: String,
    pub name: String,
    pub slug: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProjectMetadata {
    #[serde(alias = "project_id")]
    graph_id: String,
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for ProjectMetadata {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("must parse")
    }
}
