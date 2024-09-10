/// Tracing target for logging
pub const GRAFBASE_TARGET: &str = "grafbase";

/// Cache span
pub mod cache;
/// GraphQL span
pub mod graphql;
/// Request span
pub mod http_request;
/// Resolver span
pub mod resolver;
/// Subgraph span
pub mod subgraph;

/// Extension trait to record resolver invocation attributes
pub trait ResolverInvocationRecorderSpanExt {
    /// Recording error details in the span
    fn record_failure(&self, error: &str);
}

/// Extension trait to record cache operation attributes
pub trait CacheRecorderSpanExt {
    /// Recording cache status in the span
    fn record_status(&self, value: http::HeaderValue);
    /// Recording cached operation as error
    fn record_error(&self);
}
