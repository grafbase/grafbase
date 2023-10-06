use reqwest::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("the provided connection string is not a valid url: {}", _0)]
    InvalidConnectionString(String),
    #[error("request timeout")]
    Timeout,
    #[error("error sending request")]
    Request,
    #[error("error connecting to server: {}", _0)]
    ServiceUnavailable(String),
    #[error("authentication failure: {}", _0)]
    Unauthorized(String),
    #[error("internal error: {}", _0)]
    Internal(String),
    #[error("error code {}: {}", code, message)]
    Query { code: String, message: String },
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
