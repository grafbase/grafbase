use http::Response;
use http_body::Body;

/// Tracing target for logging
pub const GRAFBASE_TARGET: &str = "grafbase";

/// Cache span
pub mod cache;
/// GraphQL span
pub mod graphql;
/// Request span
pub mod request;
/// Resolver span
pub mod resolver;
/// Subgraph span
pub mod subgraph;

/// Extension trait to record http response attributes
pub trait HttpRecorderSpanExt {
    /// Recording response attributes in the span
    fn record_response<B: Body>(&self, response: &Response<B>);
    /// Record response failure in the span
    fn record_failure(&self, error: String);
    /// Record response failure in the span
    fn record_status_code(&self, status_code: http::StatusCode);
}

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
