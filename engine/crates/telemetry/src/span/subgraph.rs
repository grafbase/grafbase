use tracing::{field::Empty, info_span, Span};
use url::Url;

use crate::graphql::SubgraphResponseStatus;

use super::{graphql::record_graphql_response_status, kind::GrafbaseSpanKind};

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
        let kind: &'static str = GrafbaseSpanKind::SubgraphGraphqlRequest.into();
        let span = info_span!(
            target: crate::span::GRAFBASE_TARGET,
            "subgraph-request",
            "grafbase.kind" = kind,
            "otel.name" = self.subgraph_name,
            "otel.kind" = "Client",
            "otel.status_code" = Empty,
            "subgraph.name" = self.subgraph_name,
            "graphql.operation.type" = self.operation_type,
            "graphql.operation.document" = self.sanitized_query,
            // "Describes a class of error the operation ended with."
            "error.type" = Empty,
            // Response
            "graphql.response.data.is_present"  = Empty,
            "graphql.response.data.is_null"  = Empty,
            "graphql.response.errors.count" = Empty,
            "graphql.response.errors.distinct_codes" = Empty,
        );
        SubgraphGraphqlRequestSpan { span }
    }
}

#[derive(Clone)]
pub struct SubgraphHttpRequestSpan {
    span: Span,
}

impl std::ops::Deref for SubgraphHttpRequestSpan {
    type Target = Span;
    fn deref(&self) -> &Self::Target {
        &self.span
    }
}

impl SubgraphHttpRequestSpan {
    pub fn new(url: &Url, method: &http::Method) -> Self {
        let span = info_span!(
            target: crate::span::GRAFBASE_TARGET,
            "subgraph-http-request",
            "otel.name" = format!("{} {}", method.as_str(), url.path()),
            "http.request.method" = method.as_str(),
            "server.address" = url.host_str(),
            "server.port" = url.port(),
            "url.full" = url.as_str(),
            "http.response.status_code" = Empty,
            "otel.status_code" = Empty,
            "error.type" = Empty,
            "http.request.resend_count" = Empty,
        );

        Self { span }
    }

    pub fn record_http_status_code(&self, status_code: http::StatusCode) {
        self.record("http.response.status_code", status_code.as_u16());

        if !status_code.is_success() {
            self.record("otel.status_code", "Error");
            self.record("error.type", status_code.as_str());
        }
    }

    pub fn record_resend_count(&self, count: usize) {
        if count > 0 {
            self.record("http.request.resend_count", count);
        }
    }

    pub fn set_as_http_error(&self, status_code: Option<http::StatusCode>) {
        if let Some(status_code) = status_code {
            self.record_http_status_code(status_code);
        }
        self.record("otel.status_code", "Error");
    }

    pub fn span(&self) -> Span {
        self.span.clone()
    }
}

impl SubgraphGraphqlRequestSpan {
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
