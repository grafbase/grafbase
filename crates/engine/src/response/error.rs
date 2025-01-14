pub(crate) mod code;
mod path;

pub(crate) use code::*;
use operation::Location;
pub(crate) use path::*;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub(crate) struct GraphqlError {
    pub message: Cow<'static, str>,
    pub code: ErrorCode,
    pub locations: Vec<Location>,
    pub path: Option<ErrorPath>,
    // Serialized as a map, but kept as a Vec for efficiency.
    pub extensions: Vec<(Cow<'static, str>, serde_json::Value)>,
}

impl GraphqlError {
    pub fn invalid_subgraph_response() -> Self {
        GraphqlError::new(
            "Invalid response from subgraph",
            ErrorCode::SubgraphInvalidResponseError,
        )
    }

    pub fn new(message: impl Into<Cow<'static, str>>, code: ErrorCode) -> Self {
        GraphqlError {
            message: message.into(),
            code,
            locations: Vec::new(),
            path: None,
            extensions: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_location(mut self, location: Location) -> Self {
        self.locations.push(location);
        self
    }

    #[must_use]
    pub fn with_locations(mut self, locations: impl IntoIterator<Item = Location>) -> Self {
        self.locations.extend(locations);
        self
    }

    #[must_use]
    pub fn with_path(mut self, path: impl Into<ErrorPath>) -> Self {
        self.path = Some(path.into());
        self
    }

    #[must_use]
    pub fn with_extension(mut self, key: impl Into<Cow<'static, str>>, value: impl Into<serde_json::Value>) -> Self {
        let key = key.into();
        debug_assert!(key != "code");
        self.extensions.push((key, value.into()));
        self
    }
}

impl From<runtime::error::PartialGraphqlError> for GraphqlError {
    fn from(err: runtime::error::PartialGraphqlError) -> Self {
        GraphqlError {
            message: err.message,
            code: err.code.into(),
            extensions: err.extensions,
            locations: Vec::new(),
            path: None,
        }
    }
}

impl From<operation::VariableError> for GraphqlError {
    fn from(err: operation::VariableError) -> Self {
        GraphqlError::new(err.to_string(), ErrorCode::VariableError).with_location(err.location())
    }
}

impl std::fmt::Display for GraphqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}
