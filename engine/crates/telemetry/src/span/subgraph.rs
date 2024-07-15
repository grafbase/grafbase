use tracing::{info_span, Span};
use url::Url;

/// Subgraph request span name
pub const SUBGRAPH_SPAN_NAME: &str = "subgraph";

/// Attribute key under which the subgraph name is recorded
pub const SUBGRAPH_NAME_ATTRIBUTE: &str = "subgraph.name";

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
            "subgraph.name" = self.name,
            "subgraph.url" = self.url.as_str(),
            "gql.operation.type" = self.operation_type,
            "gql.operation.query" = self.sanitized_query,
            "otel.name" = format!("{SUBGRAPH_SPAN_NAME}:{}", self.name),
        )
    }
}
