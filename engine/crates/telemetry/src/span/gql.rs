use tracing::{info_span, Span};

use crate::{
    gql_response_status::{GraphqlResponseStatus, SubgraphResponseStatus},
    span::{GqlRecorderSpanExt, GqlRequestAttributes, GqlResponseAttributes},
};

use super::SubgraphResponseAttributes;

/// The name of the GraphQL span
pub const GRAPHQL_SPAN_NAME: &str = "graphql";

/// Attribute key under which the gql operation name is recorded
pub const GRAPHQL_OPERATION_NAME_ATTRIBUTE: &str = "gql.operation.name";

/// A span for a graphql request
#[derive(Default)]
pub struct GqlRequestSpan;

impl GqlRequestSpan {
    /// Consume self and turn into a [Span]
    pub fn create() -> Span {
        use tracing::field::Empty;

        info_span!(
            target: crate::span::GRAFBASE_TARGET,
            GRAPHQL_SPAN_NAME,
            "otel.name"  = GRAPHQL_SPAN_NAME,
            "gql.operation.name"  = Empty,
            "gql.operation.type"  = Empty,
            "gql.operation.query"  = Empty,
            "gql.response.status"  = Empty,
            "gql.response.field_errors_count"  = Empty,
            "gql.response.data_is_null"  = Empty,
            "gql.response.request_errors_count"  = Empty,
        )
    }
}

impl GqlRecorderSpanExt for Span {
    fn record_gql_request(&self, attributes: GqlRequestAttributes<'_>) {
        if let Some(name) = attributes.operation_name {
            self.record("gql.operation.name", name);
            self.record("otel.name", name);
        }
        if let Some(query) = attributes.sanitized_query {
            self.record("gql.operation.query", query);
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

    fn record_subgraph_response(&self, attributes: SubgraphResponseAttributes) {
        match attributes.status {
            SubgraphResponseStatus::GraphqlResponse(status) => {
                self.record_gql_response(GqlResponseAttributes { status })
            }
            SubgraphResponseStatus::HttpError | SubgraphResponseStatus::InvalidResponseError => {
                self.record("gql.response.status", attributes.status.as_str());
            }
        }
    }
}
