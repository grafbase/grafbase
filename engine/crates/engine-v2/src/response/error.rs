use std::collections::HashMap;

use super::ResponsePath;
use crate::request::Pos;

#[derive(Debug, Default)]
pub struct GraphqlError {
    pub message: String,
    pub locations: Vec<Pos>,
    pub path: Option<ResponsePath>,
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, serde::Serialize)]
pub struct ServerError {
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub locations: Vec<Pos>,
    pub message: String,
}
