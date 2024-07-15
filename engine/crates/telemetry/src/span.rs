use std::time::Duration;

use http::Response;
use http_body::Body;

use crate::gql_response_status::{GraphqlResponseStatus, SubgraphResponseStatus};

/// Tracing target for logging
pub const GRAFBASE_TARGET: &str = "grafbase";

/// Cache span
pub mod cache;
/// GraphQL span
pub mod gql;
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

/// Extension trait to record gql request attributes
pub trait GqlRecorderSpanExt {
    /// Record GraphQL request attributes in the span
    fn record_gql_request(&self, attributes: GqlRequestAttributes<'_>);
    /// Record GraphQL response attributes in the span
    fn record_gql_response(&self, attributes: GqlResponseAttributes);
    /// Record subgraph response attributes in the span
    fn record_subgraph_response(&self, attributes: SubgraphResponseAttributes);

    fn record_gql_error(&self, error: String);
    fn record_gql_duration(&self, duration: Duration);

    fn record_gql_status(&self, status: GraphqlResponseStatus, duration: Duration, error: Option<String>) {
        self.record_gql_response(GqlResponseAttributes { status });
        self.record_gql_duration(duration);

        if let Some(e) = error {
            self.record_gql_error(e)
        }
    }

    fn record_subgraph_status(&self, status: SubgraphResponseStatus, duration: Duration, error: Option<String>) {
        self.record_subgraph_response(SubgraphResponseAttributes { status });
        self.record_gql_duration(duration);

        if let Some(e) = error {
            self.record_gql_error(e)
        }
    }
}

/// Wraps attributes of a graphql request intended to be recorded
#[derive(Debug)]
pub struct GqlRequestAttributes<'a> {
    /// GraphQL operation type
    pub operation_type: &'static str,
    /// GraphQL operation name
    pub operation_name: Option<&'a str>,
    /// Must NOT contain any sensitive data
    pub sanitized_query: Option<&'a str>,
}

/// Wraps attributes of a graphql response intended to be recorded
pub struct GqlResponseAttributes {
    pub status: GraphqlResponseStatus,
}

/// Wraps attributes of a subgraph response intended to be recorded
pub struct SubgraphResponseAttributes {
    pub status: SubgraphResponseStatus,
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
