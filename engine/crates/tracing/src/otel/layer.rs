use std::time::Duration;

use opentelemetry::trace::noop::NoopTracer;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::export::trace::SpanExporter;
use opentelemetry_sdk::runtime::RuntimeChannel;
use opentelemetry_sdk::trace::{
    BatchConfigBuilder, BatchSpanProcessor, Builder, RandomIdGenerator, Sampler, TracerProvider,
};
use opentelemetry_sdk::Resource;
use tracing::Subscriber;
use tracing_subscriber::filter::{FilterExt, Filtered};
use tracing_subscriber::layer::Filter;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{reload, EnvFilter, Layer};

use crate::config::{TracingBatchExportConfig, TracingConfig};
use crate::error::TracingError;

/// A type erased layer
pub type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;
/// A type erased layer filter
pub type BoxedFilter<S> = Box<dyn Filter<S> + Send + Sync + 'static>;
/// Wrapper type for a filter layer over erased layer and filter
pub type FilteredLayer<S> = Filtered<BoxedLayer<S>, BoxedFilter<S>, S>;

/// Creates a new OTEL tracing layer that doesn't collect or export any tracing data.
/// The main reason this exists is to act as a placeholder in the subscriber. It's wrapped in a [`reload::Layer`]
/// enabling its replacement.
// Note: this returns a `FilteredLayer` because of https://github.com/tokio-rs/tracing/issues/1629
// it could very well just return `BoxedLayer<S>` and the handler for reload would just work with `.replace()` without panicking
pub fn new_noop<S>() -> (reload::Layer<FilteredLayer<S>, S>, reload::Handle<FilteredLayer<S>, S>)
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(NoopTracer::new())
        .boxed()
        .with_filter(FilterExt::boxed(EnvFilter::new("off")));

    let (otel_layer, reload_handle) = reload::Layer::new(otel_layer);

    (otel_layer, reload_handle)
}

/// Creates a new OTEL tracing layer that uses a [`BatchSpanProcessor`] to collect and export traces
pub fn new_batched<S, R>(
    service_name: impl Into<String>,
    config: TracingConfig,
    runtime: R,
) -> Result<BoxedLayer<S>, TracingError>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
    R: RuntimeChannel,
{
    let provider = new_provider(service_name, &config, runtime)?;
    let tracer = provider.tracer("batched-otel");

    let _ = global::set_tracer_provider(provider);

    Ok(tracing_opentelemetry::layer().with_tracer(tracer).boxed())
}

/// Creates a new OTEL tracing provider.
pub fn new_provider<R>(
    service_name: impl Into<String>,
    config: &TracingConfig,
    runtime: R,
) -> Result<TracerProvider, TracingError>
where
    R: RuntimeChannel,
{
    let service_name = service_name.into();

    let builder = opentelemetry_sdk::trace::TracerProvider::builder().with_config(
        opentelemetry_sdk::trace::config()
            .with_sampler(Sampler::TraceIdRatioBased(config.sampling))
            .with_id_generator(RandomIdGenerator::default())
            .with_max_events_per_span(config.collect.max_events_per_span as u32)
            .with_max_attributes_per_span(config.collect.max_attributes_per_span as u32)
            .with_max_events_per_span(config.collect.max_events_per_span as u32)
            .with_resource(Resource::new(vec![KeyValue::new("service.name", service_name)])),
    );

    Ok(setup_exporters(builder, config, runtime)?.build())
}

fn setup_exporters<R>(
    mut tracer_provider_builder: Builder,
    config: &TracingConfig,
    runtime: R,
) -> Result<Builder, TracingError>
where
    R: RuntimeChannel,
{
    // stdout
    if let Some(stdout_exporter) = &config.exporters.stdout {
        let span_processor = build_batched_span_processor(
            stdout_exporter.timeout,
            &stdout_exporter.batch_export,
            opentelemetry_stdout::SpanExporter::default(),
            runtime.clone(),
        );
        tracer_provider_builder = tracer_provider_builder.with_span_processor(span_processor);
    }

    // otlp
    #[cfg(feature = "otlp")]
    if let Some(ref otlp_exporter_config) = config.exporters.otlp {
        use opentelemetry_otlp::{SpanExporterBuilder, WithExportConfig};
        use std::borrow::Cow;
        use std::str::FromStr;
        use tonic::metadata::MetadataKey;
        use tonic::transport::ClientTlsConfig;

        use crate::config::TracingOtlpExporterProtocol;

        let span_exporter = {
            let exporter_timeout = Duration::from_secs(otlp_exporter_config.timeout.num_seconds() as u64);

            match otlp_exporter_config.protocol {
                TracingOtlpExporterProtocol::Grpc => {
                    let grpc_config = otlp_exporter_config
                        .grpc
                        .as_ref()
                        .map(Cow::Borrowed)
                        .unwrap_or_default();

                    let metadata = {
                        // note: I'm not using MetadataMap::from_headers due to `http` crate version issues.
                        // we're using 1 but otel currently pins tonic to an older version that requires 0.2.
                        // once versions get aligned we can replace the following
                        // let headers = grpc_config.headers.try_into_map()?;

                        let mut metadata =
                            tonic::metadata::MetadataMap::with_capacity(grpc_config.headers.inner().len());

                        for (header_key, header_value) in grpc_config.headers.inner() {
                            let key = MetadataKey::from_str(header_key.as_str())
                                .map_err(|e| TracingError::SpanExporterSetup(e.to_string()))?;

                            let value = header_value
                                .to_str()
                                .map_err(|e| TracingError::SpanExporterSetup(e.to_string()))?
                                .parse()
                                .unwrap();

                            metadata.insert(key, value);
                        }

                        metadata
                    };

                    let mut grpc_exporter = opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint(otlp_exporter_config.endpoint.to_string())
                        .with_timeout(exporter_timeout)
                        .with_metadata(metadata);

                    if let Some(ref tls_config) = grpc_config.tls {
                        grpc_exporter = grpc_exporter.with_tls_config(ClientTlsConfig::try_from(tls_config.clone())?);
                    }

                    SpanExporterBuilder::from(grpc_exporter)
                        .build_span_exporter()
                        .map_err(|err| TracingError::SpanExporterSetup(err.to_string()))?
                }
                TracingOtlpExporterProtocol::Http => {
                    let http_config = otlp_exporter_config
                        .http
                        .as_ref()
                        .map(Cow::Borrowed)
                        .unwrap_or_default();

                    let http_exporter = opentelemetry_otlp::new_exporter()
                        .http()
                        .with_endpoint(otlp_exporter_config.endpoint.to_string())
                        .with_timeout(exporter_timeout)
                        .with_headers(http_config.headers.clone().try_into_map()?);

                    SpanExporterBuilder::from(http_exporter)
                        .build_span_exporter()
                        .map_err(|err| TracingError::SpanExporterSetup(err.to_string()))?
                }
            }
        };

        let span_processor = build_batched_span_processor(
            otlp_exporter_config.timeout,
            &otlp_exporter_config.batch_export,
            span_exporter,
            runtime,
        );
        tracer_provider_builder = tracer_provider_builder.with_span_processor(span_processor);
    }

    Ok(tracer_provider_builder)
}

fn build_batched_span_processor<R>(
    timeout: chrono::Duration,
    config: &TracingBatchExportConfig,
    exporter: impl SpanExporter + 'static,
    runtime: R,
) -> BatchSpanProcessor<R>
where
    R: RuntimeChannel,
{
    BatchSpanProcessor::builder(exporter, runtime)
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_max_concurrent_exports(config.max_concurrent_exports)
                .with_max_export_batch_size(config.max_export_batch_size)
                .with_max_export_timeout(Duration::from_secs(timeout.num_seconds() as u64))
                .with_max_queue_size(config.max_queue_size)
                .with_scheduled_delay(Duration::from_secs(config.scheduled_delay.num_seconds() as u64))
                .build(),
        )
        .build()
}
