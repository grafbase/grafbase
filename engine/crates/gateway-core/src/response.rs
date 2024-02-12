use std::sync::Arc;

use http::status::StatusCode;

/// Consumers of gateway_core should implement this trait for their Response types
/// to allow gateway_core to create responses
pub trait ConstructableResponse: Sized + Send {
    type Error;

    fn error(code: StatusCode, message: &str) -> Self;
    fn engine(response: Arc<engine::Response>, headers: http::HeaderMap) -> Result<Self, Self::Error>;
    fn admin(response: async_graphql::Response) -> Result<Self, Self::Error>;
}
