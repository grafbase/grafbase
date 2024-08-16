use std::{
    future::Future,
    task::{ready, Context, Poll},
    time::Instant,
};

use ::tower::{Layer, Service};
use headers::HeaderMapExt;
use http::{Request, Response};
use http_body::Body;
use opentelemetry::{metrics::Meter, propagation::Extractor};
use pin_project_lite::pin_project;
use tracing::Span;

use crate::{
    gql_response_status::GraphqlResponseStatus,
    grafbase_client::Client,
    metrics::{RequestMetrics, RequestMetricsAttributes},
    span::{request::HttpRequestSpan, GqlRecorderSpanExt, HttpRecorderSpanExt, GRAFBASE_TARGET},
};

pub fn layer(meter: Meter) -> TelemetryLayer {
    TelemetryLayer {
        metrics: RequestMetrics::build(&meter),
    }
}

#[derive(Clone)]
pub struct TelemetryLayer {
    metrics: RequestMetrics,
}

impl<S> Layer<S> for TelemetryLayer {
    type Service = TelemetryService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        TelemetryService {
            inner,
            metrics: self.metrics.clone(),
        }
    }
}

/// tower-http provides a TraceService as a convenient way to wrap the whole execution. However
/// it's only meant for tracing and doesn't provide a good way for metrics to access both the
/// request and the response. As such we end up needing to write a [tower::Service] ourselves.
/// [TelemetryService] is mostly inspired by how the [tower_http::trace::Trace] works.
#[derive(Clone)]
pub struct TelemetryService<S> {
    inner: S,
    metrics: RequestMetrics,
}

impl<S> TelemetryService<S> {
    #[cfg(not(feature = "lambda"))]
    fn make_span<B: Body>(&mut self, request: &Request<B>) -> Span {
        HttpRequestSpan::from_http(request).into_span()
    }

    #[cfg(feature = "lambda")]
    fn make_span<B: Body>(&self, request: &Request<B>) -> Span {
        use opentelemetry::Context;
        use tracing_opentelemetry::OpenTelemetrySpanExt;

        let parent_ctx = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract_with_context(&Context::current(), &HeaderExtractor(request.headers()))
        });

        let span = HttpRequestSpan::from_http(request).into_span();
        span.set_parent(parent_ctx);

        span
    }
}

// From opentelemetry-http which still uses http 0.X as of 2024/05/17
#[cfg_attr(not(feature = "lambda"), allow(unused))]
struct HeaderExtractor<'a>(pub &'a http::HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    /// Get a value for a key from the HeaderMap.  If the value is not valid ASCII, returns None.
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    /// Collect all the keys from the HeaderMap.
    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|value| value.as_str()).collect::<Vec<_>>()
    }
}

/// See [TelemetryService]
impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for TelemetryService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    S::Error: std::fmt::Display + 'static,
    ReqBody: Body,
    ResBody: Body,
{
    type Response = http::Response<ResBody>;

    type Error = S::Error;

    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let start = Instant::now();
        let client = Client::extract_from(req.headers());
        let metrics = self.metrics.clone();
        let span = self.make_span(&req);

        metrics.increment_connected_clients();

        ResponseFuture {
            inner: self.inner.call(req),
            metrics,
            span,
            start,
            client,
        }
    }
}

pin_project! {
    pub struct ResponseFuture<F> {
        #[pin]
        inner: F,
        metrics: RequestMetrics,
        span: Span,
        start: Instant,
        client: Option<Client>,
    }
}

/// See [TelemetryService]
impl<F, ResBody, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
    ResBody: Body,
    E: std::fmt::Display + 'static,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();

        let mut result = ready!(this.inner.poll(cx));
        let latency = this.start.elapsed();

        let client = this.client.take();
        let metrics = this.metrics;

        match result {
            Ok(ref mut response) => {
                let cache_status = response
                    .headers()
                    .get("x-grafbase-cache")
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_string);

                Span::current().record("http.response.status_code", response.status().as_u16());

                let gql_status = response.headers().typed_get();

                metrics.record_http_duration(
                    RequestMetricsAttributes {
                        status_code: response.status().as_u16(),
                        cache_status,
                        gql_status,
                        client,
                    },
                    latency,
                );

                match gql_status {
                    Some(status) if status.is_success() => {
                        Span::current().record_gql_status(status);
                        tracing::debug!(target: GRAFBASE_TARGET, "gateway response");
                    }
                    Some(status) => {
                        Span::current().record_gql_status(status);
                        tracing::debug!(target: GRAFBASE_TARGET, "responding a GraphQL error");
                    }
                    None => {
                        let status = GraphqlResponseStatus::RequestError { count: 1 };
                        Span::current().record_gql_status(status);

                        tracing::debug!(target: GRAFBASE_TARGET, "responding a GraphQL error");
                    }
                }

                response.headers_mut().remove(GraphqlResponseStatus::header_name());

                if let Some(size) = response.body().size_hint().exact() {
                    metrics.record_response_body_size(size);
                }
            }
            Err(ref err) => {
                Span::current().record("http.response.status_code", 500);

                metrics.record_http_duration(
                    RequestMetricsAttributes {
                        status_code: 500,
                        client,
                        cache_status: None,
                        gql_status: None,
                    },
                    latency,
                );

                Span::current().record_failure(err.to_string());
                tracing::error!(target: GRAFBASE_TARGET, "{err}");
            }
        }

        metrics.decrement_connected_clients();

        Poll::Ready(result)
    }
}
