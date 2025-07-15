use tokio::sync::{mpsc, watch};

/// The Grafbase gateway error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error validating federated SDL: {0}")]
    /// The GraphQL schema validation
    SchemaValidationError(String),
    /// Internal error
    #[error("internal error: {0}")]
    InternalError(String),
    /// Cannot find the certificate or key file
    #[error("reading certificate files: {0}")]
    CertificateError(#[source] std::io::Error),
    /// Cannot start the HTTP server
    #[error("starting server: {0}")]
    Server(#[source] std::io::Error),
    #[error("fetcher configuration error: {0}")]
    FetcherConfigError(String),
    #[error(transparent)]
    CreateExtensionCatalogError(#[from] crate::extensions::Error),
}

impl<T> From<watch::error::SendError<T>> for Error {
    fn from(value: watch::error::SendError<T>) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl<T> From<mpsc::error::SendError<T>> for Error {
    fn from(value: mpsc::error::SendError<T>) -> Self {
        Self::InternalError(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::InternalError(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::InternalError(value.to_string())
    }
}
