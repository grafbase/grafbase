use tracing::{field::Empty, info_span, Span};
use url::Url;

/// Subgraph request span name
pub const SUBGRAPH_SPAN_NAME: &str = "subgraph";

/// A span for a subgraph request
pub struct SubgraphRequestSpan<'a> {
    pub name: &'a str,
    pub operation_type: &'a str,
    pub sanitized_query: &'a str,
    pub url: &'a Url,
}

impl<'a> SubgraphRequestSpan<'a> {
    pub fn into_span(self) -> Span {
        info_span!(
            target: crate::span::GRAFBASE_TARGET,
            SUBGRAPH_SPAN_NAME,
            "otel.name" = format!("{SUBGRAPH_SPAN_NAME}:{}", self.name),
            "subgraph.name" = self.name,
            "subgraph.url" = self.url.as_str(),
            "gql.operation.type" = self.operation_type,
            "gql.operation.query" = self.sanitized_query,
            "gql.response.status" = Empty,
            "gql.response.field_errors_count" = Empty,
            "gql.response.data_is_null" = Empty,
            "gql.response.request_errors_count" = Empty,
            "gql.response.error" = Empty,
        )
    }
}
