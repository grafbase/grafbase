#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Cache(runtime::cache::Error),
    #[error(transparent)]
    Ratelimit(#[from] runtime::rate_limiting::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<gateway_core::Error> for Error {
    fn from(err: gateway_core::Error) -> Self {
        match err {
            gateway_core::Error::BadRequest(msg) => Self::BadRequest(msg),
            gateway_core::Error::Cache(err) => Self::Cache(err),
            gateway_core::Error::Serialization(msg) => Self::Serialization(msg),
        }
    }
}
