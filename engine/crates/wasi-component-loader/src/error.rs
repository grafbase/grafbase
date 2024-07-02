pub mod guest;

/// The error type from a WASI call
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error on initialization or mishandling of WASI components.
    #[error("{0}")]
    Internal(#[from] anyhow::Error),
    /// User-thrown error of the WASI guest
    #[error("{0}")]
    User(#[from] guest::Error),
}

impl Error {
    /// Converts into user error response, if one.
    pub fn into_user_error(self) -> Option<guest::Error> {
        match self {
            Error::Internal(_) => None,
            Error::User(error) => Some(error),
        }
    }
}
