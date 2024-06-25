use std::collections::BTreeMap;

use engine::ErrorCode;
use runtime::hooks::HookError;

use super::ResponsePath;

#[derive(Debug, Clone, Default)]
pub(crate) struct GraphqlError {
    pub message: String,
    pub locations: Vec<crate::operation::Location>,
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

    pub fn with_error_code(mut self, code: ErrorCode) -> Self {
        self.extensions
            .insert("code".to_string(), serde_json::Value::String(code.to_string()));
        self
    }

    pub fn internal_server_error() -> Self {
        GraphqlError::new("Internal server error").with_error_code(ErrorCode::InternalServerError)
    }
}

impl From<HookError> for GraphqlError {
    fn from(error: HookError) -> Self {
        let (message, extensions) = match error {
            HookError::User(error) => (error.message, error.extensions),
            HookError::Internal(error) => (error.to_string(), Default::default()),
        };

        Self {
            message,
            extensions,
            ..Default::default()
        }
    }
}
