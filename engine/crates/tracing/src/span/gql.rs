use tracing::{info_span, Span};

use crate::{
    gql_response_status::GraphqlResponseStatus,
    span::{GqlRecorderSpanExt, GqlRequestAttributes, GqlResponseAttributes},
};

/// The name of the GraphQL span
pub const GRAPHQL_SPAN_NAME: &str = "graphql";

/// Attribute key under which the gql operation name is recorded
pub const GRAPHQL_OPERATION_NAME_ATTRIBUTE: &str = "gql.operation.name";

/// A span for a graphql request
#[derive(Default)]
pub struct GqlRequestSpan<'a> {
    /// The operation name from the graphql query
    operation_name: Option<&'a str>,
    /// The GraphQL operation type
    operation_type: Option<&'a str>,
    /// The GraphQL query
    document: Option<&'a str>,

    // -- response --
    status: Option<&'a str>,
    field_errors_count: Option<u64>,
    data_is_null: Option<bool>,
    request_errors_count: Option<u64>,
}

impl<'a> GqlRequestSpan<'a> {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            operation_name: None,
            operation_type: None,
            document: None,
            status: None,
            field_errors_count: None,
            data_is_null: None,
            request_errors_count: None,
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
            "gql.operation.name" = self.operation_name,
            "gql.operation.type" = self.operation_type,
            "gql.document" = self.document,
            "gql.response.status" = self.status,
            "gql.response.field_errors_count" = self.field_errors_count,
            "gql.response.data_is_null" = self.data_is_null,
            "gql.response.request_errors_count" = self.request_errors_count,
        )
    }
}

impl GqlRecorderSpanExt for Span {
    fn record_gql_request(&self, attributes: GqlRequestAttributes) {
        if let Some(name) = attributes.operation_name {
            self.record("gql.operation.name", name);
        }
        self.record("gql.operation.type", attributes.operation_type);
    }

    fn record_gql_response(&self, attributes: GqlResponseAttributes) {
        self.record("gql.response.status", attributes.status.as_str());
        match attributes.status {
            GraphqlResponseStatus::Success => {}
            GraphqlResponseStatus::FieldError { count, data_is_null } => {
                self.record("gql.response.field_errors_count", count);
                self.record("gql.response.data_is_null", data_is_null);
            }
            GraphqlResponseStatus::RequestError { count } => {
                self.record("gql.response.request_errors_count", count);
            }
        }
    }
}
