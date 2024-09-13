use itertools::Itertools;
use tracing::{info_span, Span};

use crate::{
    graphql::{GraphqlOperationAttributes, GraphqlResponseStatus, OperationName},
    span::kind::GrafbaseSpanKind,
};

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

        let kind: &'static str = GrafbaseSpanKind::GraphqlOperation.into();
        let span = info_span!(
            target: crate::span::GRAFBASE_TARGET,
            "graphql",
            "grafbase.kind" = kind,
            "otel.name"  = Empty,
            "otel.kind" = "Server",
            "graphql.operation.name"  = Empty,
            "grafbase.operation.computed_name" = Empty,
            "graphql.operation.type"  = Empty,
            "graphql.operation.document"  = Empty,
            "graphql.response.data.is_present"  = Empty,
            "graphql.response.data.is_null"  = Empty,
            "graphql.response.errors.count" = Empty,
            "graphql.response.errors.distinct_codes" = Empty,
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

    pub fn record_distinct_error_codes(&self, distinct_error_codes: impl IntoIterator<Item: std::fmt::Display>) {
        self.record(
            "graphql.response.errors.distinct_codes",
            distinct_error_codes.into_iter().join(","),
        );
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
