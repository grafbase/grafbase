use std::{borrow::Cow, fmt};

/// User facing GraphQL error that will be extended with the right path & location if relevant by
/// the engine.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphqlError {
    pub message: Cow<'static, str>,
    /// Optional extensions added to the response
    /// Will be serialized as a map, but we store it as a Vec for efficiency
    pub extensions: Vec<(Cow<'static, str>, serde_json::Value)>,
}

impl GraphqlError {
    pub fn new(message: impl Into<String>) -> Self {
        GraphqlError {
            message: Cow::Owned(message.into()),
            extensions: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_extension(mut self, key: impl Into<Cow<'static, str>>, value: impl Into<serde_json::Value>) -> Self {
        self.extensions.push((key.into(), value.into()));
        self
    }

    pub fn internal_server_error() -> Self {
        GraphqlError {
            message: Cow::Borrowed("Internal server error"),
            extensions: Vec::new(),
        }
    }
}

impl From<String> for GraphqlError {
    fn from(message: String) -> Self {
        GraphqlError {
            message: Cow::Owned(message),
            extensions: Vec::new(),
        }
    }
}

impl From<&'static str> for GraphqlError {
    fn from(message: &'static str) -> Self {
        GraphqlError {
            message: Cow::Borrowed(message),
            extensions: Vec::new(),
        }
    }
}

impl fmt::Display for GraphqlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}
