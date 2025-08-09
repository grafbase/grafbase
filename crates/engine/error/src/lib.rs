mod code;
mod path;

pub use code::*;
use operation::Location;
pub use path::*;
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct ErrorResponse {
    pub status: http::StatusCode,
    pub errors: Vec<GraphqlError>,
    pub headers: http::HeaderMap,
}

impl ErrorResponse {
    pub fn new(status: http::StatusCode) -> Self {
        ErrorResponse {
            status,
            errors: Vec::new(),
            headers: Default::default(),
        }
    }

    pub fn with_error(mut self, error: GraphqlError) -> Self {
        self.errors.push(error);
        self
    }

    pub fn with_errors<I>(mut self, errors: I) -> Self
    where
        I: IntoIterator<Item = GraphqlError>,
    {
        self.errors.extend(errors);
        self
    }

    pub fn with_headers(mut self, headers: http::HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    pub fn into_message(self) -> Cow<'static, str> {
        self.errors
            .into_iter()
            .map(|err| err.message)
            .next()
            .unwrap_or_else(|| self.status.canonical_reason().unwrap_or("Internal server error").into())
    }

    pub fn internal_extension_error() -> Self {
        Self::new(http::StatusCode::INTERNAL_SERVER_ERROR).with_error(GraphqlError::internal_extension_error())
    }
}

impl From<GraphqlError> for ErrorResponse {
    fn from(error: GraphqlError) -> Self {
        ErrorResponse {
            status: error.code.into(),
            errors: vec![error],
            headers: Default::default(),
        }
    }
}

pub type GraphqlResult<T> = Result<T, GraphqlError>;

#[derive(Debug, Clone)]
pub struct GraphqlError {
    pub message: Cow<'static, str>,
    pub code: ErrorCode,
    pub locations: Vec<Location>,
    pub path: Option<ErrorPath>,
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
    pub fn with_path(mut self, path: impl Into<ErrorPath>) -> Self {
        self.path = Some(path.into());
        self
    }

    #[must_use]
    pub fn with_extension(mut self, key: impl Into<Cow<'static, str>>, value: impl Into<serde_json::Value>) -> Self {
        let key = key.into();
        self.extensions.push((key, value.into()));
        self
    }

    #[must_use]
    pub fn with_extensions(
        mut self,
        extensions: impl IntoIterator<Item = (impl Into<Cow<'static, str>>, impl Into<serde_json::Value>)>,
    ) -> Self {
        for (key, value) in extensions {
            self.extensions.push((key.into(), value.into()));
        }
        self
    }

    // ------------- //
    // Common errors //
    // ------------- //

    pub fn invalid_subgraph_response() -> Self {
        GraphqlError::new(
            "Invalid response from subgraph",
            ErrorCode::SubgraphInvalidResponseError,
        )
    }

    pub fn unauthenticated() -> Self {
        GraphqlError::new("Unauthenticated", ErrorCode::Unauthenticated)
    }

    pub fn unauthorized() -> Self {
        GraphqlError::new("Not authorized", ErrorCode::Unauthorized)
    }

    pub fn internal_server_error() -> Self {
        GraphqlError::new("Internal server error", ErrorCode::InternalServerError)
    }

    pub fn internal_extension_error() -> Self {
        GraphqlError::new("Internal extension error", ErrorCode::ExtensionError)
    }
}

impl std::fmt::Display for GraphqlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}
