pub(crate) mod code;
use std::borrow::Cow;

pub(crate) use code::*;

use crate::operation::Location;

use super::ResponsePath;

#[derive(Debug, Clone)]
pub(crate) struct GraphqlError {
    pub message: Cow<'static, str>,
    pub code: ErrorCode,
    pub locations: Vec<Location>,
    pub path: Option<ResponsePath>,
    // Serialized as a map, but kept as a Vec for efficiency.
    pub extensions: Vec<(Cow<'static, str>, serde_json::Value)>,
}

impl GraphqlError {
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
    pub fn with_path(mut self, path: ResponsePath) -> Self {
        self.path = Some(path);
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

impl std::fmt::Display for GraphqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}
