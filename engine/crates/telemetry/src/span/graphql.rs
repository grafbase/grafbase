use tracing::{info_span, Span};

use crate::graphql::{GraphqlOperationAttributes, GraphqlResponseStatus, OperationName};

/// The name of the GraphQL span
pub const GRAPHQL_SPAN_NAME: &str = "graphql";

/// Attribute key under which the gql operation name is recorded
pub const GRAPHQL_OPERATION_NAME_ATTRIBUTE: &str = "gql.operation.name";

/// A span for a graphql request
pub struct GraphqlOperationSpan {
    pub span: Span,
}

impl std::ops::Deref for GraphqlOperationSpan {
    type Target = Span;
    fn deref(&self) -> &Self::Target {
        &self.span
    }
}

impl Default for GraphqlOperationSpan {
    fn default() -> Self {
        use tracing::field::Empty;

        let span = info_span!(
            target: crate::span::GRAFBASE_TARGET,
            GRAPHQL_SPAN_NAME,
            "otel.name"  = Empty,
            "graphql.operation.name"  = Empty,
            "grafbase.operation.computed_name" = Empty,
            "graphql.operation.type"  = Empty,
            "graphql.operation.document"  = Empty,
            "graphql.response.data.is_present"  = Empty,
            "graphql.response.data.is_null"  = Empty,
            "graphql.response.errors.count" = Empty,
        );
        GraphqlOperationSpan { span }
    }
}

impl GraphqlOperationSpan {
    pub fn record_operation(&self, operation: &GraphqlOperationAttributes) {
        match &operation.name {
            OperationName::Original(name) => {
                self.record("graphql.operation.name", name);
                self.record("otel.name", name);
            }
            OperationName::Computed(name) => {
                self.record("grafbase.operation.computed_name", name);
                self.record("otel.name", name);
            }
            OperationName::Unknown => {}
        }
        self.record("graphql.operation.document", operation.sanitized_query.as_ref());
        self.record("graphql.operation.type", operation.ty.as_str());
    }

    pub fn record_response_status(&self, status: GraphqlResponseStatus) {
        record_graphql_response_status(&self.span, status);
    }
}

pub(crate) fn record_graphql_response_status(span: &Span, status: GraphqlResponseStatus) {
    match status {
        GraphqlResponseStatus::Success => {
            span.record("graphql.response.data.is_present", true);
        }
        GraphqlResponseStatus::RefusedRequest => {}
        GraphqlResponseStatus::FieldError { count, data_is_null } => {
            span.record("graphql.response.errors.count", count);
            span.record("graphql.response.data.is_present", true);
            if data_is_null {
                span.record("graphql.response.data.is_null", true);
            }
        }
        GraphqlResponseStatus::RequestError { count } => {
            span.record("graphql.response.errors.count", count);
        }
    }
}
