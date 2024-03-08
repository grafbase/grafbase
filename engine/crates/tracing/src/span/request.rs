use std::net::IpAddr;

use crate::span::HttpRecorderSpanExt;
use http::header::USER_AGENT;
use http::Response;
use http_body::Body;
use tracing::{info_span, Span};

/// The name of the span that represents the root of an incoming request
pub const SPAN_NAME: &str = "gateway";
pub(crate) const X_FORWARDED_FOR_HEADER: &str = "X-Forwarded-For";

/// A span for a http request
pub struct HttpRequestSpan<'a> {
    /// The size of the request payload body in bytes
    request_body_size: Option<usize>,
    /// HTTP request method
    request_method: &'a http::Method,
    /// The size of the response payload body in bytes
    response_body_size: Option<usize>,
    /// HTTP response status code
    response_status_code: Option<&'a http::StatusCode>,
    /// HTTP response error
    response_error: Option<&'a str>,
    /// Value of the HTTP User-Agent header sent by the client
    header_user_agent: Option<&'a http::HeaderValue>,
    /// If the request has an X-ForwardedFor header, this will have the first value that is a valid address and not private/internal
    header_x_forwarded_for: Option<&'a http::HeaderValue>,
    /// Address of the local HTTP server that received the request
    server_address: Option<&'a IpAddr>,
    /// Port of the local HTTP server that received the request
    server_port: Option<u16>,
    /// The URI of the request
    url: &'a http::Uri,
}

impl<'a> HttpRequestSpan<'a> {
    /// Create a new instance
    pub fn new<B>(request: &'a http::Request<B>) -> Self
    where
        B: Body,
    {
        HttpRequestSpan {
            request_body_size: request.body().size_hint().upper().map(|v| v as usize),
            request_method: request.method(),
            header_user_agent: request.headers().get(USER_AGENT),
            header_x_forwarded_for: request.headers().get(X_FORWARDED_FOR_HEADER),
            url: request.uri(),
            response_body_size: None,
            response_status_code: None,
            response_error: None,
            server_address: None,
            server_port: None,
        }
    }

    /// Consume self and turn into a [Span]
    pub fn into_span(self) -> Span {
        info_span!(
            target: crate::span::GRAFBASE_TARGET,
            SPAN_NAME,
            "http.request.body.size" = self.request_body_size,
            "http.request.method" = self.request_method.as_str(),
            "http.response.body.size" = self.response_body_size,
            "http.response.status_code" = self.response_status_code.map(|v| v.as_u16()),
            "http.response.error" = self.response_error,
            "http.header.user_agent" = self.header_user_agent.and_then(|v| v.to_str().ok()),
            "http.header.x_forwarded_for" = self.header_x_forwarded_for.and_then(|v| v.to_str().ok()),
            "server.address" = self.server_address.map(|v| v.to_string()),
            "server.port" = self.server_port,
            "url.path" = self.url.path(),
            "url.query" = self.url.query(),
            "url.scheme" = self.url.scheme().map(|v| v.as_str()),
        )
    }
}

#[cfg(feature = "tower")]
/// Type that implements [tower_http::trace::MakeSpan] to integrate with tower layer's
#[derive(Clone)]
pub struct MakeHttpRequestSpan;
#[cfg(feature = "tower")]
impl<B: Body> tower_http::trace::MakeSpan<B> for MakeHttpRequestSpan {
    fn make_span(&mut self, request: &http::Request<B>) -> Span {
        HttpRequestSpan::new(request).into_span()
    }
}

impl HttpRecorderSpanExt for Span {
    fn record_response<B: Body>(&self, response: &Response<B>) {
        self.record(
            "http.response.body.size",
            response.body().size_hint().upper().unwrap_or(0),
        );
        self.record("http.response.status_code", response.status().as_str());
    }

    fn record_failure(&self, error: &str) {
        self.record("http.response.error", error);
    }
}
