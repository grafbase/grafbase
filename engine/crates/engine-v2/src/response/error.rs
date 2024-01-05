use std::collections::BTreeMap;

use super::ResponsePath;
use crate::request::Pos;

#[derive(Debug, Default)]
pub(crate) struct GraphqlError {
    pub message: String,
    pub locations: Vec<Pos>,
    pub path: Option<ResponsePath>,
    // ensures consistent ordering for tests
    pub extensions: BTreeMap<String, serde_json::Value>,
}

impl GraphqlError {
    pub fn new(message: impl Into<String>) -> Self {
        GraphqlError {
            message: message.into(),
            ..Default::default()
        }
    }
}
