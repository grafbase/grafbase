use std::borrow::Cow;
use std::net::IpAddr;

use crate::grafbase_client::Client;
use crate::span::HttpRecorderSpanExt;
use http::header::USER_AGENT;
use http::{Response, StatusCode};
use http_body::Body;
use tracing::{info_span, Span};

/// The name of the span that represents the root of an incoming request
pub const GATEWAY_SPAN_NAME: &str = "gateway";
pub(crate) const X_FORWARDED_FOR_HEADER: &str = "X-Forwarded-For";

/// A span for a http request
pub struct HttpRequestSpan<'a> {
    /// The size of the request payload body in bytes
    request_body_size: Option<usize>,
    /// HTTP request method
    request_method: Cow<'a, http::Method>,
    /// The size of the response payload body in bytes
    response_body_size: Option<usize>,
    /// HTTP response status code
    response_status_code: Option<Cow<'a, http::StatusCode>>,
    /// HTTP response error
    response_error: Option<Cow<'a, str>>,
    /// Value of the HTTP User-Agent header sent by the client
    header_user_agent: Option<Cow<'a, http::HeaderValue>>,
    /// If the request has an X-ForwardedFor header, this will have the first value that is a valid address and not private/internal
    header_x_forwarded_for: Option<Cow<'a, http::HeaderValue>>,
    /// Value of the ray-id header sent by the server
    header_ray_id: Option<Cow<'a, http::HeaderValue>>,
    header_x_grafbase_client: Option<Client>,
    /// Address of the local HTTP server that received the request
    server_address: Option<Cow<'a, IpAddr>>,
    /// Port of the local HTTP server that received the request
    server_port: Option<u16>,
    /// The URI of the request
    url: Cow<'a, http::Uri>,
    /// The git branch this deployment belongs to
    git_branch: Option<Cow<'a, http::HeaderValue>>,
    /// The git hash this deployment corresponds to
    git_hash: Option<Cow<'a, http::HeaderValue>>,
    /// The environment this deployment belongs to
    environment: Option<Cow<'a, http::HeaderValue>>,
}

impl<'a> HttpRequestSpan<'a> {
    /// Sets the span ray_id
    pub fn with_ray_id(mut self, ray_id: impl Into<Option<Cow<'a, http::HeaderValue>>>) -> Self {
        self.header_ray_id = ray_id.into();

        self
    }

    /// Sets the span git_branch
    pub fn with_git_branch(mut self, git_branch: impl Into<Option<Cow<'a, http::HeaderValue>>>) -> Self {
        self.git_branch = git_branch.into();

        self
    }

    /// Sets the span git_hash
    pub fn with_git_hash(mut self, git_hash: impl Into<Option<Cow<'a, http::HeaderValue>>>) -> Self {
        self.git_hash = git_hash.into();

        self
    }

    /// Sets the span environment
    pub fn with_environment(mut self, environment: impl Into<Option<Cow<'a, http::HeaderValue>>>) -> Self {
        self.environment = environment.into();

        self
    }
}

impl<'a> HttpRequestSpan<'a> {
    /// Create a new instance from a reference of [http::Request]
    pub fn from_http<B>(request: &'a http::Request<B>) -> Self
    where
        B: Body,
    {
        HttpRequestSpan {
            request_body_size: request.body().size_hint().upper().map(|v| v as usize),
            request_method: Cow::Borrowed(request.method()),
            header_user_agent: request.headers().get(USER_AGENT).map(Cow::Borrowed),
            header_x_forwarded_for: request.headers().get(X_FORWARDED_FOR_HEADER).map(Cow::Borrowed),
            header_x_grafbase_client: Client::extract_from(request.headers()),
            header_ray_id: None,
            url: Cow::Borrowed(request.uri()),
            response_body_size: None,
            response_status_code: None,
            response_error: None,
            server_address: None,
            server_port: None,
            environment: None,
            git_branch: None,
            git_hash: None,
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

        Ok(HttpRequestSpan {
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
            response_body_size: None,
            response_status_code: None,
            response_error: None,
            server_address: None,
            server_port: None,
            environment: None,
            git_branch: None,
            git_hash: None,
        })
    }

    /// Consume self and turn into a [Span]
    pub fn into_span(self) -> Span {
        info_span!(
            target: crate::span::GRAFBASE_TARGET,
            GATEWAY_SPAN_NAME,
            "http.request.body.size" = self.request_body_size,
            "http.request.method" = self.request_method.as_str(),
            "http.response.body.size" = self.response_body_size,
            "http.response.status_code" = self.response_status_code.map(|v| v.as_u16()),
            "http.response.error" = self.response_error.as_ref().map(|v| v.as_ref()),
            "http.header.user_agent" = self.header_user_agent.as_ref().and_then(|v| v.to_str().ok()),
            "http.header.x_forwarded_for" = self.header_x_forwarded_for.as_ref().and_then(|v| v.to_str().ok()),
            "http.header.x-grafbase-client-name" = self.header_x_grafbase_client.as_ref().map(|client| client.name.as_str()),
            "http.header.x-grafbase-client-version" = self.header_x_grafbase_client.as_ref().and_then(|client| client.version.as_deref()),
            "http.header.ray_id" = self.header_ray_id.as_ref().and_then(|v| v.to_str().ok()),
            "server.address" = self.server_address.map(|v| v.to_string()),
            "server.port" = self.server_port,
            "url.path" = self.url.path(),
            "url.scheme" = self.url.scheme().map(|v| v.as_str()),
            "url.host" = self.url.host(),
            "git.branch" = self.git_branch.as_ref().and_then(|v| v.to_str().ok()),
            "git.hash" = self.git_hash.as_ref().and_then(|v| v.to_str().ok()),
            "environment" = self.environment.as_ref().and_then(|v| v.to_str().ok()),
        )
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

    fn record_failure(&self, error: String) {
        self.record("http.response.status_code", "500");
        self.record("http.response.error", error);
    }

    fn record_status_code(&self, status_code: StatusCode) {
        self.record("http.response.status_code", status_code.as_str());
    }
}
