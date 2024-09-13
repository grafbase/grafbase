use std::borrow::{Borrow, Cow};

use crate::grafbase_client::Client;
use crate::graphql::GraphqlExecutionTelemetry;
use http::header::{HOST, USER_AGENT};
use http::{Response, StatusCode};
use http_body::Body;
use itertools::Itertools;
use tracing::field::Empty;
use tracing::{info_span, Span};

use super::kind::GrafbaseSpanKind;

pub(crate) const X_FORWARDED_FOR_HEADER: &str = "X-Forwarded-For";

/// A span for a http request
pub struct HttpRequestSpanBuilder<'a> {
    /// The size of the request payload body in bytes
    request_body_size: Option<usize>,
    /// HTTP request method
    request_method: Cow<'a, http::Method>,
    /// Value of the HTTP User-Agent header sent by the client
    header_user_agent: Option<Cow<'a, http::HeaderValue>>,
    /// If the request has an X-ForwardedFor header, this will have the first value that is a valid address and not private/internal
    header_x_forwarded_for: Option<Cow<'a, http::HeaderValue>>,
    /// Value of the ray-id header sent by the server
    header_ray_id: Option<Cow<'a, http::HeaderValue>>,
    header_x_grafbase_client: Option<Client>,
    /// Address of the local HTTP server that received the request
    server_address: Option<Cow<'a, http::HeaderValue>>,
    /// Port of the local HTTP server that received the request
    server_port: Option<u16>,
    /// The URI of the request
    url: Cow<'a, http::Uri>,
}

#[derive(Clone)]
pub struct HttpRequestSpan {
    pub span: Span,
}

impl std::ops::Deref for HttpRequestSpan {
    type Target = Span;
    fn deref(&self) -> &Self::Target {
        &self.span
    }
}

impl Borrow<Span> for HttpRequestSpan {
    fn borrow(&self) -> &Span {
        &self.span
    }
}

impl<'a> HttpRequestSpanBuilder<'a> {
    /// Sets the span ray_id
    pub fn with_ray_id(mut self, ray_id: impl Into<Option<Cow<'a, http::HeaderValue>>>) -> Self {
        self.header_ray_id = ray_id.into();

        self
    }

    /// Create a new instance from a reference of [http::Request]
    pub fn from_http<B>(request: &'a http::Request<B>) -> Self
    where
        B: Body,
    {
        HttpRequestSpanBuilder {
            request_body_size: request.body().size_hint().upper().map(|v| v as usize),
            request_method: Cow::Borrowed(request.method()),
            header_user_agent: request.headers().get(USER_AGENT).map(Cow::Borrowed),
            header_x_forwarded_for: request.headers().get(X_FORWARDED_FOR_HEADER).map(Cow::Borrowed),
            header_x_grafbase_client: Client::extract_from(request.headers()),
            header_ray_id: None,
            url: Cow::Borrowed(request.uri()),
            server_address: request.headers().get(HOST).map(Cow::Borrowed),
            server_port: None,
        }
    }

    #[cfg(feature = "worker")]
    /// Create a new instance from a reference of [worker::Request]
    pub fn try_from_worker(request: &'a worker::Request) -> worker::Result<Self> {
        use core::str::FromStr;
        use http::HeaderValue;

        use crate::grafbase_client::{X_GRAFBASE_CLIENT_NAME, X_GRAFBASE_CLIENT_VERSION};

        let method =
            http::Method::from_str(request.method().as_ref()).map_err(|e| worker::Error::RustError(e.to_string()))?;

        let user_agent = request
            .headers()
            .get(USER_AGENT.as_str())?
            .and_then(|value| HeaderValue::from_str(&value).ok())
            .map(Cow::Owned);

        let x_forwarded_for = request
            .headers()
            .get(X_FORWARDED_FOR_HEADER)?
            .and_then(|value| HeaderValue::from_str(&value).ok())
            .map(Cow::Owned);

        let uri = http::Uri::try_from(request.url()?.as_str()).map_err(|e| worker::Error::RustError(e.to_string()))?;

        Ok(HttpRequestSpanBuilder {
            request_body_size: None,
            request_method: Cow::Owned(method),
            header_user_agent: user_agent,
            header_x_forwarded_for: x_forwarded_for,
            header_x_grafbase_client: Client::maybe_new(
                request.headers().get(X_GRAFBASE_CLIENT_NAME.as_str()).ok().flatten(),
                request.headers().get(X_GRAFBASE_CLIENT_VERSION.as_str()).ok().flatten(),
            ),
            header_ray_id: None,
            url: Cow::Owned(uri),
            server_address: None,
            server_port: None,
        })
    }

    /// Consume self and turn into a [Span]
    pub fn build(self) -> HttpRequestSpan {
        // We follow the HTTP server span conventions:
        // https://opentelemetry.io/docs/specs/semconv/http/http-spans/#http-server
        let kind: &'static str = GrafbaseSpanKind::HttpRequest.into();
        let span = info_span!(
            target: crate::span::GRAFBASE_TARGET,
            "http-request",
            "grafbase.kind" = kind,
            "otel.status_code" = Empty,
            "otel.kind" = "Server",
            "otel.name" = format!("{} {}", self.request_method, self.url.path()),
            // "Describes a class of error the operation ended with."
            "error.type" = Empty,
            "server.address" = self.server_address.as_ref().and_then(|v| v.to_str().ok()),
            "server.port" = self.server_port,
            "url.path" = self.url.path(),
            "url.scheme" = self.url.scheme().map(|v| v.as_str()),
            // Request
            "http.request.body.size" = self.request_body_size,
            "http.request.method" = self.request_method.as_str(),
            "user_agent.original" = self.header_user_agent.as_ref().and_then(|v| v.to_str().ok()),
            "http.request.header.x-forwarded-for" = self.header_x_forwarded_for.as_ref().and_then(|v| v.to_str().ok()),
            "http.request.header.x-grafbase-client-name" = self.header_x_grafbase_client.as_ref().map(|client| client.name.as_str()),
            "http.request.header.x-grafbase-client-version" = self.header_x_grafbase_client.as_ref().and_then(|client| client.version.as_deref()),
            // Response
            "http.response.status_code" = Empty,
            "http.response.body.size" = Empty,
            "http.response.header.ray_id" = self.header_ray_id.as_ref().and_then(|v| v.to_str().ok()),
            "graphql.operations.name" = Empty,
            "graphql.operations.type" = Empty,
            "graphql.response.errors.count" = Empty,
            "graphql.response.errors.distinct_codes" = Empty,
        );
        HttpRequestSpan { span }
    }
}

impl HttpRequestSpan {
    pub fn record_response<B: Body>(&self, response: &Response<B>) {
        self.record("http.response.status_code", response.status().as_str());
        if let Some(size) = response.body().size_hint().exact() {
            self.record("http.response.body.size", size);
        }
        if response.status().is_server_error() {
            self.record("otel.status_code", "Error");
            self.record("error.type", response.status().as_str());
        } else {
            self.record("otel.status_code", "Ok");
        }
    }

    pub fn record_graphql_execution_telemetry<ErrorCode: std::fmt::Display>(
        &self,
        telemetry: &GraphqlExecutionTelemetry<ErrorCode>,
    ) {
        self.record(
            "graphql.operations.name",
            telemetry.operations.iter().map(|(_, name)| name).join(","),
        );
        self.record(
            "graphql.operations.type",
            telemetry.operations.iter().map(|(ty, _)| ty).join(","),
        );
        self.record("graphql.response.errors.count", telemetry.errors_count);
        self.record(
            "graphql.response.errors.distinct_codes",
            telemetry.distinct_error_codes.iter().join(","),
        );
    }

    pub fn record_internal_server_error(&self) {
        self.record("otel.status_code", "Error");
        self.record("error.type", "500");
        self.record("http.response.status_code", "500");
    }

    // Used in workers
    pub fn record_status_code(&self, status_code: StatusCode) {
        self.record("http.response.status_code", status_code.as_str());
    }
}
