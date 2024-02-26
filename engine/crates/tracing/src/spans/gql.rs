use crate::spans::GqlRecorderSpanExt;
use tracing::{info_span, Span};

/// A span for a graphql request
#[derive(Default)]
pub struct GqlRequestSpan<'a> {
    /// The operation name from the graphql query
    operation_name: Option<&'a str>,
    /// query|mutation|subscription
    operation_type: Option<&'a str>,
    /// The GraphQL query
    document: Option<&'a str>,
}

impl<'a> GqlRequestSpan<'a> {
    pub fn new() -> Self {
        Default::default()
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
            "gql_request",
            "gql.request.operation.name" = self.operation_name,
            "gql.request.operation.type" = self.operation_type,
            "gql.document" = self.document,
        )
    }
}

impl GqlRecorderSpanExt for Span {
    fn record_gql_operation_type<'a>(&self, operation_type: impl Into<Option<&'a str>>) {
        self.record("gql.request.operation.type", operation_type.into());
    }
}
