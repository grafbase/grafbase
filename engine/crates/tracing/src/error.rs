#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error(transparent)]
    FileReadError(std::io::Error),
    #[error("unable to configure span exporter: {0}")]
    SpanExporterSetup(String),
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(String),
}
