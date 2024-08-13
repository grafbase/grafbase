use http::Response;
use http_body::Body;

use crate::{
    gql_response_status::{GraphqlResponseStatus, SubgraphResponseStatus},
    metrics::OperationMetricsAttributes,
};

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

    fn record_gql_status(&self, status: GraphqlResponseStatus) {
        self.record_gql_response(GqlResponseAttributes { status });
    }

    fn record_subgraph_status(&self, status: SubgraphResponseStatus) {
        self.record_subgraph_response(SubgraphResponseAttributes { status });
    }
}

/// Wraps attributes of a graphql request intended to be recorded
#[derive(Debug)]
pub struct GqlRequestAttributes<'a> {
    /// GraphQL operation type
    pub operation_type: &'static str,
    /// GraphQL operation name
    pub operation_name: Option<&'a str>,
    /// OTEL name of the span
    pub otel_name: &'a str,
    /// Must NOT contain any sensitive data
    pub sanitized_query: Option<&'a str>,
}

impl<'a> From<&'a OperationMetricsAttributes> for GqlRequestAttributes<'a> {
    fn from(metrics_attributes: &'a OperationMetricsAttributes) -> Self {
        Self {
            operation_type: metrics_attributes.ty.as_str(),
            operation_name: metrics_attributes.name.as_deref(),
            otel_name: &metrics_attributes.internal.operation_name_or_generated_one,
            sanitized_query: Some(&metrics_attributes.sanitized_query),
        }
    }
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
