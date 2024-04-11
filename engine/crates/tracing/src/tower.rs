use std::time::Duration;

use headers::HeaderMapExt;
use http::Response;
use http_body::Body;
use tower_http::classify::{ServerErrorsAsFailures, ServerErrorsFailureClass, SharedClassifier};
use tower_http::trace::{DefaultOnBodyChunk, DefaultOnEos, DefaultOnRequest};
use tracing::Span;

use crate::execution_metadata::{GraphqlExecutionMetadata, X_GRAFBASE_GRAPHQL_EXECUTION_METADATA};
use crate::span::GRAFBASE_TARGET;

/// A [tower::Layer] that creates a [tracing::Span] for each incoming request and removes the execution metadata
#[allow(clippy::type_complexity)]
// super friendly type yes... Writing the proper impl is really hard with all the requirements of
// axum.
pub fn layer<B: Body>() -> tower::ServiceBuilder<
    tower::layer::util::Stack<
        tower::util::MapResponseLayer<impl Fn(Response<B>) -> Response<B> + Clone>,
        tower::layer::util::Stack<
            tower_http::trace::TraceLayer<
                SharedClassifier<ServerErrorsAsFailures>,
                crate::span::request::MakeHttpRequestSpan,
                DefaultOnRequest,
                impl Fn(&Response<B>, Duration, &Span) + Clone,
                DefaultOnBodyChunk,
                DefaultOnEos,
                impl Fn(ServerErrorsFailureClass, Duration, &Span) + Clone,
            >,
            tower::layer::util::Identity,
        >,
    >,
> {
    tower::ServiceBuilder::new()
        .layer(tracing_layer::<B>())
        .map_response(|mut response: http::Response<B>| {
            response.headers_mut().remove(&X_GRAFBASE_GRAPHQL_EXECUTION_METADATA);
            response
        })
}

/// A [tower_http::trace::TraceLayer] that creates [crate::span::request::HttpRequestSpan] for each incoming request
/// and records request and response attributes in the span
fn tracing_layer<B: Body>() -> tower_http::trace::TraceLayer<
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
            let execution_metadata = response
                .headers()
                .typed_get::<GraphqlExecutionMetadata>()
                .unwrap_or_default();
            tracing::event!(
                target: GRAFBASE_TARGET,
                tracing::Level::INFO,
                counter.request_count = 1,
                http.response.status_code = response.status().as_u16(),
                http.response.headers.cache_status = cache_status,
                gql.request.operation.id = execution_metadata.operation_id(),
                gql.request.operation.name = execution_metadata.operation_name(),
                gql.response.has_errors = execution_metadata.has_errors(),
            );
            tracing::event!(
                target: GRAFBASE_TARGET,
                tracing::Level::INFO,
                histogram.latency = latency.as_millis() as u64,
                gql.request.operation.id = execution_metadata.operation_id(),
                gql.request.operation.name = execution_metadata.operation_name(),
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
