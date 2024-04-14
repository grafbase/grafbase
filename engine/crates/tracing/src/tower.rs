use std::time::Duration;

use http::Response;
use http_body::Body;
use opentelemetry::metrics::Meter;
use tower_http::classify::{ServerErrorsAsFailures, ServerErrorsFailureClass, SharedClassifier};
use tower_http::trace::{DefaultOnBodyChunk, DefaultOnEos, DefaultOnRequest};
use tracing::Span;

use crate::metrics::{RequestMetrics, RequestMetricsAttributes};

/// A [tower_http::trace::TraceLayer] that creates [crate::span::request::HttpRequestSpan] for each incoming request
/// and records request and response attributes in the span
pub fn layer<B: Body>(
    meter: Meter,
) -> tower_http::trace::TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    crate::span::request::MakeHttpRequestSpan,
    DefaultOnRequest,
    impl Fn(&Response<B>, Duration, &Span) + Clone,
    DefaultOnBodyChunk,
    DefaultOnEos,
    impl Fn(ServerErrorsFailureClass, Duration, &Span) + Clone,
> {
    let metrics = RequestMetrics::build(&meter);
    tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(crate::span::request::MakeHttpRequestSpan)
        .on_response({
            let metrics = metrics.clone();
            move |response: &Response<_>, latency: Duration, span: &Span| {
                use crate::span::HttpRecorderSpanExt;

                let cache_status = response
                    .headers()
                    .get("x-grafbase-cache")
                    .and_then(|value| value.to_str().ok());
                metrics.record(
                    RequestMetricsAttributes {
                        status_code: response.status().as_u16(),
                        cache_status: cache_status.map(|s| s.to_string()),
                    },
                    latency,
                );
                span.record_response(response);
            }
        })
        .on_failure(move |error: ServerErrorsFailureClass, latency: Duration, span: &Span| {
            use crate::span::HttpRecorderSpanExt;

            let status_code = match error {
                ServerErrorsFailureClass::StatusCode(code) => code.as_u16(),
                ServerErrorsFailureClass::Error(_) => 500,
            };
            metrics.record(
                RequestMetricsAttributes {
                    status_code,
                    cache_status: None,
                },
                latency,
            );
            span.record_failure(error.to_string().as_str());
        })
}
