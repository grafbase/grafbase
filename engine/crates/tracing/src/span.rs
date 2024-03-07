use http::Response;
use http_body::Body;

pub(crate) const GRAFBASE_TARGET: &str = "grafbase";

pub mod gql;
pub mod request;
pub mod subgraph;

/// Extension trait to record an http response
pub trait HttpRecorderSpanExt {
    fn record_response<B: Body>(&self, response: &Response<B>);
    fn record_failure(&self, error: &str);
}

/// Extension trait to record gql request attributes
pub trait GqlRecorderSpanExt {
    fn record_gql_request(&self, attributes: GqlRequestAttributes<'_>);
    fn record_gql_response(&self, attributes: GqlResponseAttributes);
}

/// Wraps attributes of a graphql request intended to be recorded
pub struct GqlRequestAttributes<'a> {
    pub operation_type: &'a str,
    pub operation_name: Option<&'a str>,
}

/// Wraps attributes of a graphql response intended to be recorded
pub struct GqlResponseAttributes {
    pub has_errors: bool,
}
