use core::fmt;

use wasmtime::component::{ComponentType, Lift};

/// The error type from a WASI call
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error on initialization or mishandling of WASI components.
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
    /// User-thrown error of the WASI guest
    #[error("{0}")]
    User(#[from] ErrorResponse),
}

impl Error {
    /// Converts into user error response, if one.
    pub fn into_user_error(self) -> Option<ErrorResponse> {
        match self {
            Error::Internal(_) => None,
            Error::User(error) => Some(error),
        }
    }
}

/// An error type available for the user to throw from the guest.
#[derive(Clone, ComponentType, Lift, Debug, thiserror::Error, PartialEq)]
#[component(record)]
pub struct ErrorResponse {
    /// Additional extensions added to the GraphQL response
    pub extensions: Vec<(String, String)>,
    /// The error message
    pub message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}
