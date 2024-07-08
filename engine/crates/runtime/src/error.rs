use std::{borrow::Cow, fmt};

/// Some of the error codes that the engine supports
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, strum::Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum PartialErrorCode {
    InternalServerError,
    BadRequest,
    Unauthorized,
    HookError,
}

/// User facing GraphQL error that will be extended with the right path & location if relevant by
/// the engine.
#[derive(Clone, Debug, PartialEq)]
pub struct PartialGraphqlError {
    pub message: Cow<'static, str>,
    pub code: PartialErrorCode,
    /// Optional extensions added to the response
    /// Will be serialized as a map, but we store it as a Vec for efficiency
    pub extensions: Vec<(Cow<'static, str>, serde_json::Value)>,
}

impl PartialGraphqlError {
    pub fn new(message: impl Into<Cow<'static, str>>, code: PartialErrorCode) -> Self {
        PartialGraphqlError {
            message: message.into(),
            code,
            extensions: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_extension(mut self, key: impl Into<Cow<'static, str>>, value: impl Into<serde_json::Value>) -> Self {
        self.extensions.push((key.into(), value.into()));
        self
    }

    pub fn internal_server_error() -> Self {
        PartialGraphqlError {
            message: Cow::Borrowed("Internal server error"),
            code: PartialErrorCode::InternalServerError,
            extensions: Vec::new(),
        }
    }

    pub fn internal_hook_error() -> Self {
        PartialGraphqlError {
            message: Cow::Borrowed("Internal hook error"),
            code: PartialErrorCode::HookError,
            extensions: Vec::new(),
        }
    }
}

impl fmt::Display for PartialGraphqlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}
