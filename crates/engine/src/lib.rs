#![deny(clippy::future_not_send, unused_crate_dependencies)]

use grafbase_workspace_hack as _;

mod engine;
mod execution;
mod graphql_over_http;
mod prepare;
mod resolver;
mod response;
mod utils;
pub mod websocket;

pub use engine::{Engine, Runtime, WebsocketSession, mcp};
pub use error::{ErrorCode, ErrorResponse, GraphqlError};
pub use graphql_over_http::{Body, ResponseFormat, TelemetryExtension};
pub use prepare::cached::CachedOperation;
pub use schema::Schema;

pub fn http_error_response(
    format: ResponseFormat,
    ErrorResponse {
        status,
        errors,
        headers,
    }: ErrorResponse,
) -> http::Response<Body> {
    graphql_over_http::Http::error(format, response::Response::refused_request(status, errors, headers))
}
