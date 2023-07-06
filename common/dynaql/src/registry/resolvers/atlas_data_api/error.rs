use http::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("the request was malformed: {0}")]
    BadRequest(String),
    #[error(
        "unauthorized API key, please ensure that your Data API key is enabled for the cluster"
    )]
    Unauthorized,
    #[error("the request was sent to an endpoint that does not exist")]
    NotFound,
    #[error("the Atlas Data API encountered an internal error and could not complete the request")]
    TargetInternalError,
    #[error("request to upstream server failed: {0}")]
    RequestError(reqwest::Error),
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        match value.status() {
            Some(StatusCode::BAD_REQUEST) => Self::BadRequest(value.to_string()),
            Some(StatusCode::UNAUTHORIZED) => Self::Unauthorized,
            Some(StatusCode::NOT_FOUND) => Self::NotFound,
            Some(StatusCode::INTERNAL_SERVER_ERROR) => Self::TargetInternalError,
            _ => Self::RequestError(value),
        }
    }
}
