pub mod guest;

/// The error type from a WASI call from the gateway.
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    /// Error on initialization or mishandling of WASI components.
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
    /// Error defined by the guest.
    #[error("{0}")]
    Guest(#[from] guest::ErrorResponse),
}

impl From<Error> for GatewayError {
    fn from(value: Error) -> Self {
        match value {
            Error::Internal(error) => GatewayError::Internal(error),
            Error::Guest(error) => GatewayError::Guest(guest::ErrorResponse {
                status_code: 500,
                errors: vec![error],
            }),
        }
    }
}

impl GatewayError {
    /// Converts into user error response, if one.
    pub fn into_guest_error(self) -> Option<guest::ErrorResponse> {
        match self {
            GatewayError::Internal(_) => None,
            GatewayError::Guest(error) => Some(error),
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
    Guest(#[from] guest::GuestError),
}

impl Error {
    /// Converts into user error response, if one.
    pub fn into_guest_error(self) -> Option<guest::GuestError> {
        match self {
            Error::Internal(_) => None,
            Error::Guest(error) => Some(error),
        }
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Internal(anyhow::anyhow!(error))
    }
}
