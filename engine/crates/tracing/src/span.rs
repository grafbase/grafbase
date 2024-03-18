use http::Response;
use http_body::Body;

pub(crate) const GRAFBASE_TARGET: &str = "grafbase";

/// GraphQL span
pub mod gql;
/// Request span
pub mod request;
/// Subgraph span
pub mod subgraph;

/// Extension trait to record http response attributes
pub trait HttpRecorderSpanExt {
    /// Recording response attributes in the span
    fn record_response<B: Body>(&self, response: &Response<B>);
    /// Record response failure in the span
    fn record_failure(&self, error: &str);
    /// Record response failure in the span
    fn record_status_code(&self, status_code: http::StatusCode);
}

/// Extension trait to record gql request attributes
pub trait GqlRecorderSpanExt {
    /// Record GraphQL request attributes in the span
    fn record_gql_request(&self, attributes: GqlRequestAttributes<'_>);
    /// Record GraphQL response attributes in the span
    fn record_gql_response(&self, attributes: GqlResponseAttributes);
}

/// Wraps attributes of a graphql request intended to be recorded
pub struct GqlRequestAttributes<'a> {
    /// GraphQL operation type
    pub operation_type: &'a str,
    /// GraphQL operation name
    pub operation_name: Option<&'a str>,
}

/// Wraps attributes of a graphql response intended to be recorded
pub struct GqlResponseAttributes {
    /// If the GraphQL response contains errors, record it in the span
    pub has_errors: bool,
}
