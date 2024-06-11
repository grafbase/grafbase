use axum::response::IntoResponse;
use engine_v2::{HttpGraphqlResponse, HttpGraphqlResponseBody};
use runtime::bytes::OwnedOrSharedBytes;

pub mod websocket;

pub fn error(message: &str) -> axum::response::Response {
    into_response(HttpGraphqlResponse::request_error(message))
}

pub fn into_response(response: HttpGraphqlResponse) -> axum::response::Response {
    let HttpGraphqlResponse { headers, body, .. } = response;

    match body {
        HttpGraphqlResponseBody::Bytes(bytes) => match bytes {
            OwnedOrSharedBytes::Owned(bytes) => (headers, bytes).into_response(),
            OwnedOrSharedBytes::Shared(bytes) => (headers, bytes).into_response(),
        },
        HttpGraphqlResponseBody::Stream(stream) => (headers, axum::body::Body::from_stream(stream)).into_response(),
    }
}
