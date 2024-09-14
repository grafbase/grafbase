use grafbase_workspace_hack as _;

use std::sync::Arc;

use axum::{response::IntoResponse, Json};
use engine_v2::{Body, Engine, ErrorCode, Runtime};
use futures_util::TryFutureExt;
use runtime::bytes::OwnedOrSharedBytes;

pub mod middleware;
pub mod websocket;

pub fn internal_server_error(message: impl ToString) -> axum::response::Response {
    let body = Json(serde_json::json!({
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

/// Executes the engine with the given request and body limit.
///
/// # Parameters
/// - `engine`: The federated GraphQL execution engine.
/// - `request`: The incoming request to be processed.
/// - `body_limit_bytes`: The maximum allowable size of the request body in bytes.
///
/// # Returns
/// An HTTP response object containing the result of the execution.
///
/// # Errors
/// Returns an HTTP 413 status code if the request body exceeds the specified limit,
/// or a 500 status code for internal errors during execution.
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

    let (parts, body) = engine
        .execute(http::Request::from_parts(parts, body))
        .await
        .into_parts();

    match body {
        Body::Bytes(bytes) => match bytes {
            OwnedOrSharedBytes::Owned(bytes) => (parts.status, parts.headers, parts.extensions, bytes).into_response(),
            OwnedOrSharedBytes::Shared(bytes) => (parts.status, parts.headers, parts.extensions, bytes).into_response(),
        },
        Body::Stream(stream) => (
            parts.status,
            parts.headers,
            parts.extensions,
            axum::body::Body::from_stream(stream),
        )
            .into_response(),
    }
}
