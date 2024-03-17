use tracing::{info_span, Span};

use crate::span::{GqlRecorderSpanExt, GqlRequestAttributes, GqlResponseAttributes};

/// The name of the GraphQL span
pub const GRAPHQL_SPAN_NAME: &str = "graphql";

/// A span for a graphql request
#[derive(Default)]
pub struct GqlRequestSpan<'a> {
    /// True if the response contains errors
    has_errors: Option<bool>,
    /// The operation name from the graphql query
    operation_name: Option<&'a str>,
    /// The GraphQL operation type
    operation_type: Option<&'a str>,
    /// The GraphQL query
    document: Option<&'a str>,
}

impl<'a> GqlRequestSpan<'a> {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            has_errors: None,
            operation_name: None,
            operation_type: None,
            document: None,
        }
    }

    /// Set the GraphQL document as an attribute of the span
    pub fn with_document(mut self, document: impl Into<Option<&'a str>>) -> Self {
        self.document = document.into();
        self
    }

    /// Set the operation name as an attribute of the span
    pub fn with_operation_name(mut self, operation_name: impl Into<Option<&'a str>>) -> Self {
        self.operation_name = operation_name.into();
        self
    }

    /// Set the operation type as an attribute of the span
    pub fn with_operation_type(mut self, operation_type: impl Into<Option<&'a str>>) -> Self {
        self.operation_type = operation_type.into();
        self
    }

    /// Consume self and turn into a [Span]
    pub fn into_span(self) -> Span {
        info_span!(
            target: crate::span::GRAFBASE_TARGET,
            GRAPHQL_SPAN_NAME,
            "gql.request.operation.name" = self.operation_name,
            "gql.request.operation.type" = self.operation_type,
            "gql.response.has_errors" = self.has_errors,
            "gql.document" = self.document,
        )
    }
}

impl GqlRecorderSpanExt for Span {
    fn record_gql_request(&self, attributes: GqlRequestAttributes<'_>) {
        if let Some(name) = attributes.operation_name {
            self.record("gql.request.operation.name", name);
        }
        self.record("gql.request.operation.type", attributes.operation_type);
    }

    fn record_gql_response(&self, attributes: GqlResponseAttributes) {
        self.record("gql.response.has_errors", attributes.has_errors);
    }
}
