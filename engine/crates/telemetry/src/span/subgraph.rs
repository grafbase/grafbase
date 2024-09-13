use tracing::{field::Empty, info_span, Span};
use url::Url;

use crate::graphql::SubgraphResponseStatus;

use super::graphql::record_graphql_response_status;

/// Subgraph request span name
pub const SUBGRAPH_SPAN_NAME: &str = "subgraph";

/// A span for a subgraph request
pub struct SubgraphRequestSpanBuilder<'a> {
    pub subgraph_name: &'a str,
    pub operation_type: &'static str,
    pub sanitized_query: &'a str,
}

#[derive(Clone)]
pub struct SubgraphGraphqlRequestSpan {
    pub span: Span,
}

impl std::ops::Deref for SubgraphGraphqlRequestSpan {
    type Target = Span;
    fn deref(&self) -> &Self::Target {
        &self.span
    }
}

impl<'a> SubgraphRequestSpanBuilder<'a> {
    pub fn build(self) -> SubgraphGraphqlRequestSpan {
        // We follow the HTTP client span conventions:
        // https://opentelemetry.io/docs/specs/semconv/http/http-spans/#http-client
        let span = info_span!(
            target: crate::span::GRAFBASE_TARGET,
            SUBGRAPH_SPAN_NAME,
            "otel.name" = format!("{SUBGRAPH_SPAN_NAME}:{}", self.subgraph_name),
            "otel.status_code" = Empty,
            "subgraph.name" = self.subgraph_name,
            // "Describes a class of error the operation ended with."
            "error.type" = Empty,
            // Request
            "http.request.method" = Empty,
            "server.address" = Empty,
            "server.port" = Empty,
            "url.full" = Empty,
            "http.request.resend_count" = Empty,
            "graphql.operation.type" = self.operation_type,
            "graphql.operation.document" = self.sanitized_query,
            // Response
            "http.response.status_code" = Empty,
            "graphql.response.data.is_present"  = Empty,
            "graphql.response.data.is_null"  = Empty,
            "graphql.response.errors.count" = Empty,
            "graphql.response.errors.distinct_codes" = Empty,
        );
        SubgraphGraphqlRequestSpan { span }
    }
}

impl SubgraphGraphqlRequestSpan {
    pub fn record_http_request(&self, url: &Url, method: &http::Method) {
        self.record("http.request.method", method.as_str());
        self.record("server.address", url.host_str());
        self.record("server.port", url.port());
        self.record("url.full", url.as_str());
    }

    pub fn record_resend_count(&self, count: usize) {
        if count > 0 {
            self.record("http.request.resend_count", count);
        }
    }
    pub fn record_http_status_code(&self, status_code: http::StatusCode) {
        self.record("http.response.status_code", status_code.as_u16());
        if !status_code.is_success() {
            self.record("otel.status_code", "Error");
            self.record("error.type", status_code.as_str());
        }
    }

    pub fn record_graphql_response_status(&self, status: SubgraphResponseStatus) {
        match status {
            SubgraphResponseStatus::WellFormedGraphqlResponse(status) => {
                record_graphql_response_status(&self.span, status);
                if status.is_request_error() {
                    self.record("otel.status_code", "Error");
                    self.record("error.type", status.as_str());
                }
            }
            failure => {
                self.record("otel.status_code", "Error");
                self.record("error.type", failure.as_str());
            }
        }
    }
}
