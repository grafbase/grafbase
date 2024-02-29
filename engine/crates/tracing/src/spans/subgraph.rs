use tracing::{info_span, Span};

/// A span for a subgraph request
pub struct SubgraphRequestSpan<'a> {
    name: &'a str,
    operation_name: Option<&'a str>,
    operation_type: Option<&'a str>,
    document: Option<&'a str>,
}
impl<'a> SubgraphRequestSpan<'a> {
    pub fn new(name: &'a str) -> Self {
        SubgraphRequestSpan {
            name,
            operation_name: None,
            operation_type: None,
            document: None,
        }
    }

    pub fn with_document(mut self, document: impl Into<Option<&'a str>>) -> Self {
        self.document = document.into();
        self
    }

    pub fn with_operation_name(mut self, operation_name: impl Into<Option<&'a str>>) -> Self {
        self.operation_name = operation_name.into();
        self
    }

    pub fn with_operation_type(mut self, operation_type: impl Into<Option<&'a str>>) -> Self {
        self.operation_type = operation_type.into();
        self
    }

    pub fn into_span(self) -> Span {
        info_span!(
            "subgraph_request",
            "subgraph.name" = self.name,
            "subgraph.gql.operation.name" = self.operation_name.as_ref(),
            "subgraph.gql.operation.type" = self.operation_type,
            "subgraph.gql.document" = self.document,
        )
    }
}
