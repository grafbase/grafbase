use std::time::Duration;

use http::Response;
use http_body::Body;
use tower_http::classify::{ServerErrorsAsFailures, ServerErrorsFailureClass, SharedClassifier};
use tower_http::trace::{DefaultOnBodyChunk, DefaultOnEos, DefaultOnRequest};
use tracing::Span;

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
        .on_response(|response: &Response<_>, _latency: Duration, span: &Span| {
            use crate::span::HttpRecorderSpanExt;

            span.record_response(response);
        })
        .on_failure(|error: ServerErrorsFailureClass, _latency: Duration, span: &Span| {
            use crate::span::HttpRecorderSpanExt;

            span.record_failure(error.to_string().as_str());
        })
}
