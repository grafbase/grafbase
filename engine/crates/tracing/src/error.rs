/// Tracing errors
#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    /// Error reading a file from disk
    #[error(transparent)]
    FileReadError(std::io::Error),
    /// Error configuring an exporter
    #[error("unable to configure span exporter: {0}")]
    SpanExporterSetup(String),
    /// Error configuring a metric exporter
    #[error("unable to configure metrics exporter: {0}")]
    MetricsExporterSetup(String),
}

impl From<String> for TracingError {
    fn from(s: String) -> Self {
        TracingError::Internal(s)
    }
}

impl From<&str> for TracingError {
    fn from(s: &str) -> Self {
        TracingError::Internal(s.to_string())
    }
}
