use runtime::error::{PartialErrorCode, PartialGraphqlError};

use crate::extension::api::wit;

/// The error type from a WASI call from the gateway.
#[derive(Debug, thiserror::Error)]
pub enum ErrorResponse {
    /// Error on initialization or mishandling of WASI components.
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
    /// Error defined by the guest.
    #[error("{0}")]
    Guest(#[from] wit::error::ErrorResponse),
}

impl From<Error> for ErrorResponse {
    fn from(value: Error) -> Self {
        match value {
            Error::Internal(error) => ErrorResponse::Internal(error),
            Error::Guest(error) => ErrorResponse::Guest(wit::error::ErrorResponse {
                status_code: 500,
                errors: vec![error],
            }),
        }
    }
}

impl ErrorResponse {
    pub(crate) fn into_graphql_error_response(self, code: PartialErrorCode) -> runtime::error::ErrorResponse {
        match self {
            ErrorResponse::Internal(error) => runtime::error::ErrorResponse {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                errors: vec![PartialGraphqlError::new(
                    error.to_string(),
                    PartialErrorCode::InternalServerError,
                )],
            },
            ErrorResponse::Guest(error) => runtime::error::ErrorResponse {
                status: http::StatusCode::from_u16(error.status_code)
                    .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR),
                errors: error
                    .errors
                    .into_iter()
                    .map(|err| err.into_graphql_error(code))
                    .collect(),
            },
        }
    }
}

/// The error type from a WASI call.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error on initialization or mishandling of WASI components.
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
    /// User-thrown error of the WASI guest.
    #[error("{0}")]
    Guest(#[from] wit::error::Error),
}

impl Error {
    pub(crate) fn into_graphql_error(self, code: PartialErrorCode) -> PartialGraphqlError {
        match self {
            Error::Internal(error) => PartialGraphqlError::new(error.to_string(), code),
            Error::Guest(error) => error.into_graphql_error(code),
        }
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Internal(anyhow::anyhow!(error))
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Internal(value.into())
    }
}
