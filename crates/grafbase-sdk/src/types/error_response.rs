use std::borrow::Cow;

use http::StatusCode;

use crate::{SdkError, wit};

use super::Error;

/// A response containing a status code and multiple errors.
pub struct ErrorResponse(wit::ErrorResponse);

impl From<ErrorResponse> for wit::ErrorResponse {
    fn from(resp: ErrorResponse) -> Self {
        resp.0
    }
}

impl ErrorResponse {
    /// Creates a new [ErrorResponse] with the given HTTP status code.
    pub fn new(status_code: http::StatusCode) -> Self {
        Self(wit::ErrorResponse {
            status_code: status_code.as_u16(),
            errors: Vec::new(),
        })
    }

    /// Creates a new [ErrorResponse] with a 500 status code
    pub fn internal_server_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Creates a new [ErrorResponse] with a 400 status code
    pub fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST)
    }

    /// Creates a new [ErrorResponse] with a 401 status code
    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED)
    }

    /// Creates a new [ErrorResponse] with a 403 status code
    pub fn forbidden() -> Self {
        Self::new(StatusCode::FORBIDDEN)
    }

    /// Add a new error to the response and return self
    #[must_use]
    pub fn with_error(mut self, error: impl Into<Error>) -> Self {
        self.push_error(error);
        self
    }

    /// Adds a new error to the response.
    pub fn push_error(&mut self, error: impl Into<Error>) {
        self.0.errors.push(Into::<Error>::into(error).into());
    }
}

impl From<Error> for ErrorResponse {
    fn from(error: Error) -> Self {
        ErrorResponse::internal_server_error().with_error(error)
    }
}

impl From<SdkError> for ErrorResponse {
    fn from(err: SdkError) -> Self {
        ErrorResponse::internal_server_error().with_error(err)
    }
}

impl From<String> for ErrorResponse {
    fn from(err: String) -> Self {
        ErrorResponse::internal_server_error().with_error(err)
    }
}

impl From<&str> for ErrorResponse {
    fn from(err: &str) -> Self {
        ErrorResponse::internal_server_error().with_error(err)
    }
}

impl From<Cow<'_, str>> for ErrorResponse {
    fn from(err: Cow<'_, str>) -> Self {
        ErrorResponse::internal_server_error().with_error(err)
    }
}

impl From<wit::Error> for ErrorResponse {
    fn from(err: wit::Error) -> Self {
        ErrorResponse::internal_server_error().with_error(err)
    }
}
