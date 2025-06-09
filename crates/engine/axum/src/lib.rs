use std::sync::Arc;

use axum::{Json, response::IntoResponse};
use engine::{Body, Engine, ErrorCode, Runtime};
use futures_util::TryFutureExt;

/// Utilities for converting GraphQL ErrorResponse to HTTP responses.
///
/// This module provides functions to serialize `ErrorResponse` objects into proper
/// HTTP responses with JSON-formatted GraphQL errors. It respects the Accept header
/// to determine the appropriate Content-Type and status code handling according to
/// the GraphQL-over-HTTP specification.
pub mod error_response;
#[cfg(feature = "lambda")]
pub mod lambda;
pub mod middleware;
pub mod websocket;

pub fn internal_server_error(message: impl ToString) -> axum::response::Response {
    let body = Json(sonic_rs::json!({
        "errors": [
            {
                "message": message.to_string(),
                "extensions": {
                    "code": ErrorCode::InternalServerError
                }
            }
        ]
    }));

    (http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
}

pub async fn execute<R: Runtime>(
    engine: Arc<Engine<R>>,
    request: axum::extract::Request,
    body_limit_bytes: usize,
) -> axum::response::Response {
    let (parts, body) = request.into_parts();
    let body = axum::body::to_bytes(body, body_limit_bytes).map_err(|error| {
        if let Some(source) = std::error::Error::source(&error) {
            if source.is::<http_body_util::LengthLimitError>() {
                return (
                    http::StatusCode::PAYLOAD_TOO_LARGE,
                    format!("Request body exceeded: {}", body_limit_bytes),
                );
            }
        }
        (http::StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
    });

    let response = engine.execute(http::Request::from_parts(parts, body)).await;

    let (parts, body) = response.into_parts();
    match body {
        Body::Bytes(bytes) => (parts.status, parts.headers, parts.extensions, bytes).into_response(),
        Body::Stream(stream) => (
            parts.status,
            parts.headers,
            parts.extensions,
            axum::body::Body::from_stream(stream),
        )
            .into_response(),
    }
}
