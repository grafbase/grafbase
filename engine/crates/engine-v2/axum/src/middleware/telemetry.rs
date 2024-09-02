use std::{
    fmt::Display,
    future::Future,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use ::tower::Layer;
use grafbase_telemetry::{
    gql_response_status::GraphqlResponseStatus,
    grafbase_client::Client,
    metrics::{RequestMetrics, RequestMetricsAttributes},
    otel::{
        opentelemetry::{self, metrics::Meter, propagation::Extractor},
        tracing_opentelemetry,
    },
    span::{request::HttpRequestSpan, GqlRecorderSpanExt, HttpRecorderSpanExt, GRAFBASE_TARGET},
};
use headers::HeaderMapExt;
use http::{Request, Response};
use http_body::Body;
use tracing::Span;

#[derive(Clone)]
pub struct TelemetryLayer {
    metrics: RequestMetrics,
    listen_address: Option<SocketAddr>,
}

impl TelemetryLayer {
    pub fn new(meter: Meter, listen_address: Option<SocketAddr>) -> Self {
        Self {
            metrics: RequestMetrics::build(&meter),
            listen_address,
        }
    }
}

impl<Service> Layer<Service> for TelemetryLayer
where
    Service: Send + Clone,
{
    type Service = TelemetryService<Service>;

    fn layer(&self, inner: Service) -> Self::Service {
        TelemetryService {
            inner,
            metrics: self.metrics.clone(),
            listen_address: self.listen_address,
        }
    }
}

/// tower-http provides a TraceService as a convenient way to wrap the whole execution. However
/// it's only meant for tracing and doesn't provide a good way for metrics to access both the
/// request and the response. As such we end up needing to write a [tower::Service] ourselves.
/// [TelemetryService] is mostly inspired by how the [tower_http::trace::Trace] works.
#[derive(Clone)]
pub struct TelemetryService<Service>
where
    Service: Send + Clone,
{
    inner: Service,
    metrics: RequestMetrics,
    listen_address: Option<SocketAddr>,
}

impl<Service> TelemetryService<Service>
where
    Service: Send + Clone,
{
    fn make_span<B: Body>(&self, request: &Request<B>) -> Span {
        use tracing_opentelemetry::OpenTelemetrySpanExt;

        let parent_ctx = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(request.headers()))
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
impl<Service, ReqBody, ResBody> tower::Service<Request<ReqBody>> for TelemetryService<Service>
where
    Service: tower::Service<Request<ReqBody>, Response = Response<ResBody>> + Send + Clone + 'static,
    Service::Future: Send,
    Service::Error: Display + 'static,
    ReqBody: Body + Send + 'static,
    ResBody: Body + Send + 'static,
{
    type Response = http::Response<ResBody>;
    type Error = Service::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<ResBody>, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let metrics = self.metrics.clone();
        let span = self.make_span(&req);
        let listen_address = self.listen_address;
        let client = Client::extract_from(req.headers());
        let version = req.version();

        let method = req.method().clone();
        let url = req.uri().clone();

        metrics.increment_connected_clients();

        Box::pin(async move {
            let _guard = span.enter();
            let start = Instant::now();

            let mut result = inner.call(req).await;
            let latency = start.elapsed();

            match result {
                Err(ref err) => {
                    Span::current().record("http.response.status_code", 500);

                    metrics.record_http_duration(
                        RequestMetricsAttributes {
                            status_code: 500,
                            client,
                            cache_status: None,
                            gql_status: None,
                            url_scheme: url.scheme_str().map(ToString::to_string),
                            route: Some(url.path().to_string()),
                            listen_address,
                            version: Some(version),
                            method: Some(method.clone()),
                        },
                        latency,
                    );

                    Span::current().record_failure(err.to_string());
                    tracing::error!(target: GRAFBASE_TARGET, "Internal server error: {err}");
                }
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
                            url_scheme: url.scheme_str().map(ToString::to_string),
                            route: Some(url.path().to_string()),
                            listen_address,
                            version: Some(version),
                            method: Some(method),
                        },
                        latency,
                    );

                    match gql_status {
                        Some(status) if status.is_success() => {
                            Span::current().record_gql_status(status);
                        }
                        Some(status) => {
                            Span::current().record_gql_status(status);
                        }
                        None => {
                            let status = GraphqlResponseStatus::RequestError { count: 1 };
                            Span::current().record_gql_status(status);
                        }
                    }

                    response.headers_mut().remove(GraphqlResponseStatus::header_name());

                    if let Some(size) = response.body().size_hint().exact() {
                        metrics.record_response_body_size(size);
                    }
                }
            }

            result
        })
    }
}
