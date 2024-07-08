use futures_util::future::BoxFuture;
use std::net::IpAddr;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Too many requests")]
    ExceededCapacity,
    #[error("internal error: {0}")]
    Internal(String),
}

pub trait RateLimiterContext {
    fn header(&self, name: http::HeaderName) -> Option<&http::HeaderValue>;
    fn graphql_operation_name(&self) -> Option<&str>;
    fn ip(&self) -> Option<IpAddr>;
    fn jwt_claim(&self, key: &str) -> Option<&serde_json::Value>;
}

pub trait RateLimiter: Send + Sync {
    fn limit<'a>(&'a self, context: Box<dyn RateLimiterContext + 'a>) -> BoxFuture<'a, Result<(), Error>>;
}
