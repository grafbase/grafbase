use std::collections::BTreeMap;

use super::ResponsePath;

#[derive(Debug, Default)]
pub(crate) struct GraphqlError {
    pub message: String,
    pub locations: Vec<crate::request::Location>,
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
