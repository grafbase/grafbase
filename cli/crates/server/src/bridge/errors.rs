use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("user-defined function invocation error")]
    UdfInvocation,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::UdfInvocation => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
