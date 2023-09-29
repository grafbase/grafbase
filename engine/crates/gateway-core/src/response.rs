use std::sync::Arc;

use http::status::StatusCode;

pub trait Response: Sized + Send {
    type Error;

    #[must_use]
    fn with_additional_headers(self, headers: http::HeaderMap) -> Self;

    fn error(code: StatusCode, message: &str) -> Self;
    fn engine(response: Arc<engine::Response>) -> Result<Self, Self::Error>;
    fn admin(response: async_graphql::Response) -> Result<Self, Self::Error>;
}
