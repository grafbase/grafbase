pub mod guest;

/// The error type from a WASI call
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error on initialization or mishandling of WASI components.
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
    /// User-thrown error of the WASI guest
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
