use tracing::{info_span, Span};
use url::Url;

/// Subgraph request span name
pub const SUBGRAPH_SPAN_NAME: &str = "subgraph";

/// Attribute key under which the subgraph name is recorded
pub const SUBGRAPH_NAME_ATTRIBUTE: &str = "subgraph.name";

/// A span for a subgraph request
pub struct SubgraphRequestSpan<'a> {
    name: &'a str,
    operation_name: Option<&'a str>,
    operation_type: Option<&'a str>,
    document: Option<&'a str>,
    url: Option<&'a Url>,
}
impl<'a> SubgraphRequestSpan<'a> {
    /// Create a new instance
    pub fn new(name: &'a str) -> Self {
        SubgraphRequestSpan {
            name,
            operation_name: None,
            operation_type: None,
            document: None,
            url: None,
        }
    }

    /// Set the subgraph GraphQL document as an attribute of the span
    pub fn with_document(mut self, document: &'a str) -> Self {
        self.document = Some(document);
        self
    }

    /// Set the subgraph operation name as an attribute of the span
    pub fn with_operation_name(mut self, operation_name: impl Into<Option<&'a str>>) -> Self {
        self.operation_name = operation_name.into();
        self
    }

    /// Set the subgraph operation type as an attribute of the span
    pub fn with_operation_type(mut self, operation_type: &'a str) -> Self {
        self.operation_type = Some(operation_type);
        self
    }

    /// Set the subgraph url as an attribute of the span
    pub fn with_url(mut self, url: &'a Url) -> Self {
        self.url = Some(url);
        self
    }

    /// Consume self and turn into a [Span]
    pub fn into_span(self) -> Span {
        info_span!(
            target: crate::span::GRAFBASE_TARGET,
            SUBGRAPH_SPAN_NAME,
            "subgraph.name" = self.name,
            "subgraph.url" = self.url.map(|url| url.as_str()).unwrap_or_default(),
            "subgraph.gql.operation.name" = self.operation_name.as_ref(),
            "subgraph.gql.operation.type" = self.operation_type,
            "subgraph.gql.document" = self.document,
            "otel.name" = format!("{SUBGRAPH_SPAN_NAME}:{}", self.name),
        )
    }
}
