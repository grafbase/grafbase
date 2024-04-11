use tracing::{info_span, Span};

/// Subgraph request span name
pub const SUBGRAPH_SPAN_NAME: &str = "subgraph";

/// A span for a subgraph request
pub struct SubgraphRequestSpan<'a> {
    name: &'a str,
    operation_name: Option<&'a str>,
    operation_type: Option<&'a str>,
    document: Option<&'a str>,
}
impl<'a> SubgraphRequestSpan<'a> {
    /// Create a new instance
    pub fn new(name: &'a str) -> Self {
        SubgraphRequestSpan {
            name,
            operation_name: None,
            operation_type: None,
            document: None,
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

    /// Consume self and turn into a [Span]
    pub fn into_span(self) -> Span {
        info_span!(
            target: crate::span::GRAFBASE_TARGET,
            SUBGRAPH_SPAN_NAME,
            "subgraph.name" = self.name,
            "subgraph.gql.operation.name" = self.operation_name.as_ref(),
            "subgraph.gql.operation.type" = self.operation_type,
            "subgraph.gql.document" = self.document,
        )
    }
}
