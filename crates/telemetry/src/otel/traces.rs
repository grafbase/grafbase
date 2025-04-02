use std::time::Duration;

use gateway_config::{LayeredOtlExporterConfig, OtlpExporterProtocolConfig};
use opentelemetry_otlp::Protocol;
use opentelemetry_sdk::{
    Resource,
    trace::{
        self, BatchConfigBuilder, BatchSpanProcessor, IdGenerator, Sampler, SdkTracerProvider, TracerProviderBuilder,
    },
};

use crate::{
    config::{BatchExportConfig, TelemetryConfig},
    error::TracingError,
};

pub(super) fn build_trace_provider<I>(
    id_generator: I,
    config: &TelemetryConfig,
    resource: Resource,
) -> Result<SdkTracerProvider, TracingError>
where
    I: IdGenerator + 'static,
{
    let base_sampler = Sampler::TraceIdRatioBased(config.tracing.sampling);
    let mut builder = TracerProviderBuilder::default().with_id_generator(id_generator);

    if config.tracing.parent_based_sampler {
        builder = builder.with_sampler(Sampler::ParentBased(Box::new(base_sampler)));
    } else {
        builder = builder.with_sampler(base_sampler);
    }

    builder = builder
        .with_max_events_per_span(config.tracing.collect.max_events_per_span as u32)
        .with_max_attributes_per_span(config.tracing.collect.max_attributes_per_span as u32)
        .with_max_events_per_span(config.tracing.collect.max_events_per_span as u32)
        .with_resource(resource);

    Ok(setup_exporters(builder, config)?.build())
}

fn setup_exporters(
    mut tracer_provider_builder: TracerProviderBuilder,
    config: &TelemetryConfig,
) -> Result<TracerProviderBuilder, TracingError> {
    // stdout
    if let Some(stdout_exporter) = config.tracing_stdout_config() {
        let span_processor = build_batched_span_processor(
            stdout_exporter.timeout,
            &stdout_exporter.batch_export.unwrap_or_default(),
            opentelemetry_stdout::SpanExporter::default(),
        );

        tracer_provider_builder = tracer_provider_builder.with_span_processor(span_processor);
    }

    use super::exporter::{build_metadata, build_tls_config};
    use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithHttpConfig, WithTonicConfig};

    let build_otlp_exporter = |config: &LayeredOtlExporterConfig| {
        let exporter_timeout = config.timeout();

        let exporter = match config.protocol() {
            OtlpExporterProtocolConfig::Grpc(grpc_config) => SpanExporter::builder()
                .with_tonic()
                .with_endpoint(
                    config
                        .local
                        .endpoint
                        .as_ref()
                        .or(config.global.endpoint.as_ref())
                        .map(|url| url.as_str())
                        .unwrap_or("http://127.0.0.1:4317"),
                )
                .with_timeout(exporter_timeout)
                .with_metadata(build_metadata(grpc_config.headers))
                .with_tls_config(build_tls_config(grpc_config.tls)?)
                .build()
                .map_err(|e| TracingError::SpanExporterSetup(e.to_string()))?,
            OtlpExporterProtocolConfig::Http(http_config) => {
                SpanExporter::builder()
                    .with_http()
                    .with_endpoint(
                        // Imitate Opentelemetry default behavior
                        config
                            .local
                            .endpoint
                            .as_ref()
                            .map(|url| url.to_string())
                            .or(config.global.endpoint.as_ref().map(|url| {
                                let mut url = url.clone();
                                if url.path() == "/" || url.path().is_empty() {
                                    url.set_path("/v1/traces");
                                }
                                url.to_string()
                            }))
                            .unwrap_or("http://127.0.0.1:4318/v1/traces".to_string()),
                    )
                    .with_protocol(Protocol::HttpBinary)
                    .with_headers(http_config.headers.into_map())
                    .with_timeout(exporter_timeout)
                    .build()
                    .map_err(|e| TracingError::SpanExporterSetup(e.to_string()))?
            }
        };

        Result::<SpanExporter, TracingError>::Ok(exporter)
    };

    // otlp
    if let Some(otlp_exporter_config) = config.tracing_otlp_config() {
        let span_exporter = build_otlp_exporter(&otlp_exporter_config)?;

        let span_processor = build_batched_span_processor(
            otlp_exporter_config.timeout(),
            &otlp_exporter_config.batch_export(),
            span_exporter,
        );

        tracer_provider_builder = tracer_provider_builder.with_span_processor(span_processor);
    }

    if let Some(config) = config.grafbase_otlp_config() {
        let config = LayeredOtlExporterConfig {
            global: config.clone(),
            local: config.clone(),
        };
        let span_exporter = build_otlp_exporter(&config)?;

        let span_processor = build_batched_span_processor(config.timeout(), &config.batch_export(), span_exporter);

        tracer_provider_builder = tracer_provider_builder.with_span_processor(span_processor);
    }

    Ok(tracer_provider_builder)
}

fn build_batched_span_processor(
    timeout: Duration,
    config: &BatchExportConfig,
    exporter: impl trace::SpanExporter + 'static,
) -> BatchSpanProcessor {
    BatchSpanProcessor::builder(exporter)
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_max_concurrent_exports(config.max_concurrent_exports)
                .with_max_export_batch_size(config.max_export_batch_size)
                .with_max_export_timeout(timeout)
                .with_max_queue_size(config.max_queue_size)
                .with_scheduled_delay(config.scheduled_delay)
                .build(),
        )
        .build()
}
