use tokio::sync::{mpsc, watch};

/// The Grafbase gateway error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    /// The GraphQL schema validation
    SchemaValidationError(String),
    #[error("invalid graph_ref: {0}")]
    /// Provided graph_ref is invalid
    InvalidGraphRef(String),
    /// Internal error
    #[error("internal error: {0}")]
    InternalError(String),
    /// Cannot find the certificate or key file
    #[error("reading certificate files: {0}")]
    CertificateError(#[source] std::io::Error),
    /// Cannot start the HTTP server
    #[error("starting server: {0}")]
    Server(#[source] std::io::Error),
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
