/// Tracing errors
#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    /// Error reading a file from disk
    #[error(transparent)]
    FileReadError(std::io::Error),
    /// Error configuring an exporter
    #[error("unable to configure span exporter: {0}")]
    SpanExporterSetup(String),
}
