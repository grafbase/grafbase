use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not commit transaction: {0}")]
    FailedCommit(String),
    #[error("the provided connection string is not a valid url: {0}")]
    InvalidConnectionString(String),
    #[error("request timeout")]
    Timeout,
    #[error("could not send a request")]
    Request,
    #[error("could not connect to the server: {0}")]
    ServiceUnavailable(String),
    #[error("authentication failure: {0}")]
    Unauthorized(String),
    #[error("{0}")]
    Internal(String),
    #[error("could not connect to Postgres: {0}")]
    Connection(String),
    #[error("could not start a tx in Postgres: {0}")]
    Transaction(String),
    #[error("code {}: {}", code, message)]
    Query { code: String, message: String },
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[cfg(feature = "pooling")]
    #[error(transparent)]
    DeadpoolConfig(#[from] deadpool_postgres::ConfigError),
    #[cfg(feature = "pooling")]
    #[error(transparent)]
    DeadpoolBuild(#[from] deadpool_postgres::BuildError),
    #[cfg(feature = "pooling")]
    #[error("pg pool: {0}")]
    Deadpool(String),
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::InvalidConnectionString(error.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            return Self::Timeout;
        }

        if error.is_request() {
            return Self::Request;
        }

        if error.is_status() {
            match error.status() {
                Some(StatusCode::SERVICE_UNAVAILABLE) => return Self::ServiceUnavailable(error.to_string()),
                Some(StatusCode::UNAUTHORIZED) => return Self::Unauthorized(error.to_string()),
                _ => return Self::Internal(error.to_string()),
            }
        }

        Self::Internal(error.to_string())
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(error: tokio_postgres::Error) -> Self {
        match error.code() {
            Some(code) => Self::Query {
                code: code.code().to_string(),
                message: error.to_string(),
            },
            None => Self::Internal(error.to_string()),
        }
    }
}
