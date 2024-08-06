use std::borrow::Cow;

use enumset::EnumSetType;
use runtime::error::PartialErrorCode;

use crate::operation::Location;

use super::ResponsePath;

#[derive(Debug, serde::Serialize, serde::Deserialize, strum::Display, strum::AsRefStr, EnumSetType)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ErrorCode {
    BadRequest,
    InternalServerError,
    TrustedDocumentError,
    // Used for APQ
    PersistedQueryError,
    PersistedQueryNotFound,
    // Subgraph errors
    SubgraphError,
    SubgraphInvalidResponseError,
    SubgraphRequestError,
    // Auth
    Unauthenticated,
    Unauthorized,
    // Operation preparation phases
    OperationParsingError,
    OperationValidationError,
    OperationPlanningError,
    // Runtime
    HookError,
    // Rate limit
    RateLimited,
    // Timeouts
    GatewayTimeout,
}

impl ErrorCode {
    pub fn into_http_status_code_with_priority(self) -> (http::StatusCode, usize) {
        match self {
            ErrorCode::OperationParsingError
            | ErrorCode::OperationValidationError
            | ErrorCode::OperationPlanningError
            | ErrorCode::PersistedQueryNotFound
            | ErrorCode::PersistedQueryError
            | ErrorCode::TrustedDocumentError
            | ErrorCode::BadRequest => (http::StatusCode::BAD_REQUEST, 1000),
            ErrorCode::Unauthenticated => (http::StatusCode::UNAUTHORIZED, 600),
            ErrorCode::Unauthorized => (http::StatusCode::FORBIDDEN, 600),
            ErrorCode::RateLimited => (http::StatusCode::TOO_MANY_REQUESTS, 500),
            ErrorCode::SubgraphError | ErrorCode::SubgraphInvalidResponseError | ErrorCode::SubgraphRequestError => {
                (http::StatusCode::BAD_GATEWAY, 300)
            }
            ErrorCode::GatewayTimeout => (http::StatusCode::GATEWAY_TIMEOUT, 200),
            // least helpful error codes
            ErrorCode::HookError | ErrorCode::InternalServerError => (http::StatusCode::INTERNAL_SERVER_ERROR, 0),
        }
    }
}

impl From<PartialErrorCode> for ErrorCode {
    fn from(code: PartialErrorCode) -> Self {
        match code {
            PartialErrorCode::HookError => Self::HookError,
            PartialErrorCode::Unauthorized => Self::Unauthorized,
            PartialErrorCode::BadRequest => Self::BadRequest,
            _ => Self::InternalServerError,
        }
    }
}

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
