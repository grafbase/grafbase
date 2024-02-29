use opentelemetry::trace::TracerProvider;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::SpanExporterBuilder;
use opentelemetry_sdk::export::trace::SpanExporter;
use opentelemetry_sdk::runtime::RuntimeChannel;
use opentelemetry_sdk::trace::{BatchConfig, BatchMessage, BatchSpanProcessor, RandomIdGenerator, Sampler, Tracer};
use opentelemetry_sdk::Resource;
use std::time::Duration;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::reload;

/// Available tracing exporters generic over runtime (R)
pub enum TracingExporter<R> {
    Oltp(R),
    Stdout(R),
}

impl<R> TracingExporter<R>
where
    R: RuntimeChannel<BatchMessage>,
{
    pub fn into_batched_span_processor(self) -> BatchSpanProcessor<R> {
        match self {
            TracingExporter::Oltp(runtime) => {
                use opentelemetry_otlp::WithExportConfig;

                build_batched_span_processor(
                    SpanExporterBuilder::from(
                        opentelemetry_otlp::new_exporter()
                            .tonic()
                            .with_endpoint("http://localhost:4317")
                            .with_timeout(Duration::from_secs(3)), //.with_metadata()
                    )
                    .build_span_exporter()
                    .expect("should successfully build oltp span_exporter"),
                    runtime,
                )
            }
            TracingExporter::Stdout(runtime) => {
                build_batched_span_processor(opentelemetry_stdout::SpanExporter::default(), runtime)
            }
        }
    }
}

/// Creates a new OTEL tracing layer that uses a [`BatchSpanProcessor`] to collect and export traces
pub fn new_batched_layer<S>() -> OpenTelemetryLayer<S, Tracer>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let config = opentelemetry_sdk::trace::config()
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(64)
        .with_max_attributes_per_span(16)
        .with_max_events_per_span(16)
        .with_resource(Resource::new(vec![KeyValue::new("service.name", "my-service-name")]));

    let builder = opentelemetry_sdk::trace::TracerProvider::builder().with_config(config);
    let builder = [
        TracingExporter::Oltp(opentelemetry_sdk::runtime::Tokio),
        TracingExporter::Stdout(opentelemetry_sdk::runtime::Tokio),
    ]
    .into_iter()
    .fold(builder, |builder, exporter| {
        builder.with_span_processor(exporter.into_batched_span_processor())
    });

    let provider = builder.build();

    let tracer = provider.tracer("batched-otel");

    let _ = global::set_tracer_provider(provider);

    tracing_opentelemetry::layer().with_tracer(tracer)
}

/// Creates a new OTEL tracing layer that doesn't collect or export any tracing data.
/// The main reason this exists is to act as a placeholder in the subscriber. It's wrapped in a [`reload::Layer`]
/// enabling its replacement.
pub fn new_noop_layer<S>() -> (ReloadableOtelLayer<S, Tracer>, ReloadableOtelLayerHandler<S, Tracer>)
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_config(opentelemetry_sdk::trace::config().with_sampler(Sampler::AlwaysOff))
        .build();

    let tracer = provider.tracer("noop");

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let (otel_layer, reload_handle) = reload::Layer::new(otel_layer);

    (otel_layer, reload_handle)
}

/// A wrapper type for a reloadable OTEL layer
pub type ReloadableOtelLayer<S, T> = reload::Layer<OpenTelemetryLayer<S, T>, S>;
/// A wrapper type for a reloadable OTEL layer handler
pub type ReloadableOtelLayerHandler<S, T> = reload::Handle<OpenTelemetryLayer<S, T>, S>;

fn build_batched_span_processor<R>(exporter: impl SpanExporter + 'static, runtime: R) -> BatchSpanProcessor<R>
where
    R: RuntimeChannel<BatchMessage>,
{
    BatchSpanProcessor::builder(exporter, runtime)
        .with_batch_config(
            BatchConfig::default()
                .with_max_concurrent_exports(2)
                .with_max_export_batch_size(5_000)
                .with_max_export_timeout(Duration::from_secs(5))
                .with_max_queue_size(20_000)
                .with_scheduled_delay(Duration::from_secs(1)),
        )
        .build()
}

#[cfg(feature = "tower")]
pub mod tower {
    use http::Response;
    use http_body::Body;
    use std::time::Duration;
    use tower_http::classify::ServerErrorsFailureClass;
    use tower_http::trace::{DefaultOnBodyChunk, DefaultOnEos, DefaultOnRequest};
    use tracing::Span;

    pub fn tower_layer<B: Body>() -> tower_http::trace::TraceLayer<
        tower_http::trace::HttpMakeClassifier,
        crate::spans::request::MakeHttpRequestSpan,
        DefaultOnRequest,
        impl Fn(&Response<B>, Duration, &Span) + Clone,
        DefaultOnBodyChunk,
        DefaultOnEos,
        impl Fn(ServerErrorsFailureClass, Duration, &Span) + Clone,
    > {
        tower_http::trace::TraceLayer::new_for_http()
            .make_span_with(crate::spans::request::MakeHttpRequestSpan)
            .on_response(|response: &Response<_>, _latency: Duration, span: &Span| {
                use crate::spans::HttpRecorderSpanExt;

                span.record_response(response);
            })
            .on_failure(|error: ServerErrorsFailureClass, _latency: Duration, span: &Span| {
                use crate::spans::HttpRecorderSpanExt;

                span.record_failure(error.to_string().as_str());
            })
    }
}
