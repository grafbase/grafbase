use std::collections::HashMap;

use super::ResponsePath;
use crate::request::Pos;

#[derive(Debug, Default)]
pub(crate) struct GraphqlError {
    pub message: String,
    pub locations: Vec<Pos>,
    pub path: Option<ResponsePath>,
    pub extensions: HashMap<String, serde_json::Value>,
}
