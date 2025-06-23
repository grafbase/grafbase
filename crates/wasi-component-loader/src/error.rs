use engine_error::ErrorCode;
use wasmtime::Store;

use crate::extension::api::wit;

/// The error type from a WASI call from the gateway.
#[derive(Debug, thiserror::Error)]
pub enum ErrorResponse {
    /// Error on initialization or mishandling of WASI components.
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
    /// Error defined by the guest.
    #[error("status code: {status_code:?}, errors: {errors:?}, headers: {headers:?}")]
    Guest {
        status_code: http::StatusCode,
        errors: Vec<wit::Error>,
        headers: http::HeaderMap,
    },
}

impl ErrorResponse {
    pub(crate) fn from_wit(store: &mut Store<crate::state::WasiState>, error: wit::ErrorResponse) -> Self {
        let headers = if let Some(resource) = error.headers {
            match store
                .data_mut()
                .take_resource::<crate::resources::WasmOwnedOrLease<http::HeaderMap>>(resource.rep())
            {
                Ok(headers) => headers.into_inner().unwrap(),
                Err(err) => return Self::Internal(err.into()),
            }
        } else {
            Default::default()
        };

        ErrorResponse::Guest {
            status_code: http::StatusCode::from_u16(error.status_code)
                .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR),
            errors: error.errors,
            headers,
        }
    }

    pub(crate) fn into_graphql_error_response(self, code: ErrorCode) -> engine_error::ErrorResponse {
        match self {
            ErrorResponse::Internal(error) => {
                tracing::error!("Wasm error: {error}");
                engine_error::ErrorResponse::from(engine_error::GraphqlError::new(
                    "Internal error",
                    ErrorCode::ExtensionError,
                ))
            }
            ErrorResponse::Guest {
                status_code,
                errors,
                headers,
            } => engine_error::ErrorResponse {
                status: status_code,
                errors: errors.into_iter().map(|err| err.into_graphql_error(code)).collect(),
                headers,
            },
        }
    }
}

impl From<Error> for ErrorResponse {
    fn from(value: Error) -> Self {
        match value {
            Error::Internal(error) => ErrorResponse::Internal(error),
            Error::Guest(error) => ErrorResponse::Guest {
                status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
                errors: vec![error],
                headers: Default::default(),
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
    Guest(#[from] wit::Error),
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
