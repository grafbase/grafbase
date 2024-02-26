use http::Response;
use http_body::Body;

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
    fn record_gql_operation_type<'a>(&self, operation_type: impl Into<Option<&'a str>>);
}
