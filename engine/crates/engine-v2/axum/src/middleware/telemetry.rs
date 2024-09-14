use std::{
    fmt::Display,
    future::Future,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use ::tower::Layer;
use engine_v2::TelemetryExtension;
use grafbase_telemetry::{
    grafbase_client::Client,
    metrics::{RequestMetrics, RequestMetricsAttributes},
    otel::{
        opentelemetry::{self, metrics::Meter, propagation::Extractor},
        tracing_opentelemetry::OpenTelemetrySpanExt,
    },
    span::http_request::{HttpRequestSpan, HttpRequestSpanBuilder},
};
use http::{Request, Response};
use http_body::Body;
use tracing::Instrument;

#[derive(Clone)]
/// A layer for collecting telemetry metrics for HTTP requests.
///
/// This layer wraps a service and is responsible for tracking request metrics such as
/// duration, response sizes, and client connections.
pub struct TelemetryLayer {
    metrics: RequestMetrics,
    listen_address: Option<SocketAddr>,
}

impl TelemetryLayer {
    /// Creates a new instance of the `TelemetryLayer`.
    ///
    /// This function initializes the `TelemetryLayer` with a given `Meter` for reporting telemetry
    /// metrics and an optional `SocketAddr` the gateway listens on.
    ///
    /// # Arguments
    ///
    /// * `meter` - A `Meter` instance used for creating and reporting metrics.
    /// * `listen_address` - An optional socket address the gateway listens on.
    ///
    /// # Returns
    ///
    /// Returns a new `TelemetryLayer` instance configured with the provided `meter` and `listen_address`.
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
    /// Creates a telemetry span for the given HTTP request.
    ///
    /// This method constructs a `HttpRequestSpan` using the provided request. The span is
    /// set up to track the duration and details of the request for telemetry purposes.
    ///
    /// # Type Parameters
    ///
    /// * `B` - The type of the body of the HTTP request, which must implement the `Body` trait.
    ///
    /// # Arguments
    ///
    /// * `request` - A reference to the HTTP request for which the span is being created.
    ///
    /// # Returns
    ///
    /// Returns a `HttpRequestSpan` that can be used for tracing the request lifecycle.
    fn make_span<B: Body>(&self, request: &Request<B>) -> HttpRequestSpan {
        let parent_ctx = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(request.headers()))
        });

        let span = HttpRequestSpanBuilder::from_http(request).build();
        span.set_parent(parent_ctx);

        span
    }
}

// From opentelemetry-http which still uses http 0.X as of 2024/05/17
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

    /// Processes an HTTP request and returns a future that resolves to the HTTP response.
    ///
    /// This method is called when an incoming HTTP request is received. It records telemetry metrics
    /// for the request, including duration, response size, and any errors that occur during processing.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request to be processed.
    ///
    /// # Returns
    ///
    /// A future that resolves to a result containing either the HTTP response or an error.
    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let metrics = self.metrics.clone();
        let http_span = self.make_span(&req);
        let listen_address = self.listen_address;

        metrics.increment_connected_clients();

        let span = http_span.span.clone();
        let fut = async move {
            let start = Instant::now();

            let client = Client::extract_from(req.headers());
            let version = req.version();

            let method = req.method().clone();
            let url = req.uri().clone();

            let mut result = inner.call(req).await;

            match result {
                Err(ref err) => {
                    metrics.record_http_duration(
                        RequestMetricsAttributes {
                            status_code: 500,
                            client,
                            cache_status: None,
                            url_scheme: url.scheme_str().map(ToString::to_string),
                            route: Some(url.path().to_string()),
                            listen_address,
                            version: Some(version),
                            method: Some(method.clone()),
                        },
                        start.elapsed(),
                    );

                    http_span.record_internal_server_error();
                    tracing::error!("Internal server error: {err}");
                }
                Ok(ref mut response) => {
                    if let Some(size) = response.body().size_hint().exact() {
                        metrics.record_response_body_size(size);
                    }
                    http_span.record_response(response);
                    let cache_status = response
                        .headers()
                        .get("x-grafbase-cache")
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);

                    let attributes = RequestMetricsAttributes {
                        status_code: response.status().as_u16(),
                        client,
                        cache_status,
                        url_scheme: url.scheme_str().map(ToString::to_string),
                        route: Some(url.path().to_string()),
                        listen_address,
                        version: Some(version),
                        method: Some(method.clone()),
                    };

                    let telemetry = response
                        .extensions_mut()
                        .remove::<TelemetryExtension>()
                        .unwrap_or_default();

                    match telemetry {
                        TelemetryExtension::Ready(telemetry) => {
                            http_span.record_graphql_execution_telemetry(&telemetry);
                            metrics.record_http_duration(attributes, start.elapsed());
                        }
                        TelemetryExtension::Future(channel) => {
                            let metrics = metrics.clone();
                            let span = http_span.span.clone();
                            tokio::spawn(
                                async move {
                                    let telemetry = channel.await.unwrap_or_default();
                                    http_span.record_graphql_execution_telemetry(&telemetry);
                                    metrics.record_http_duration(attributes, start.elapsed());
                                }
                                // Ensures the span will have the proper end time.
                                .instrument(span),
                            );
                        }
                    }
                }
            }

            metrics.decrement_connected_clients();

            result
        };

        Box::pin(fut.instrument(span))
    }
}
