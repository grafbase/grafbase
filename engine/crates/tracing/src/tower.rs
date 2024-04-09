use std::time::Duration;

use http::Response;
use http_body::Body;
use tower_http::classify::{ServerErrorsAsFailures, ServerErrorsFailureClass, SharedClassifier};
use tower_http::trace::{DefaultOnBodyChunk, DefaultOnEos, DefaultOnRequest};
use tracing::Span;

use crate::span::GRAFBASE_TARGET;

/// A [tower_http::trace::TraceLayer] that creates [crate::span::request::HttpRequestSpan] for each incoming request
/// and records request and response attributes in the span
pub fn layer<B: Body>() -> tower_http::trace::TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    crate::span::request::MakeHttpRequestSpan,
    DefaultOnRequest,
    impl Fn(&Response<B>, Duration, &Span) + Clone,
    DefaultOnBodyChunk,
    DefaultOnEos,
    impl Fn(ServerErrorsFailureClass, Duration, &Span) + Clone,
> {
    tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(crate::span::request::MakeHttpRequestSpan)
        .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
            use crate::span::HttpRecorderSpanExt;

            let cache_status = response
                .headers()
                .get("x-grafbase-cache")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default();
            tracing::event!(
                target: GRAFBASE_TARGET,
                tracing::Level::INFO,
                counter.request_count = 1,
                http.response.status_code = response.status().as_u16(),
                http.response.headers.cache_status = cache_status
            );
            tracing::event!(
                target: GRAFBASE_TARGET,
                tracing::Level::INFO,
                histogram.latency = latency.as_millis(),
            );
            span.record_response(response);
        })
        .on_failure(|error: ServerErrorsFailureClass, _latency: Duration, span: &Span| {
            use crate::span::HttpRecorderSpanExt;

            let status = match error {
                ServerErrorsFailureClass::StatusCode(code) => code.as_u16(),
                ServerErrorsFailureClass::Error(_) => 500,
            };
            tracing::event!(
                target: GRAFBASE_TARGET,
                tracing::Level::INFO,
                counter.request = 1,
                http.response.status_code = status,
                http.response.headers.cache_stauts = "BYPASS"

            );
            span.record_failure(error.to_string().as_str());
        })
}
