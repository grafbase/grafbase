use std::time::Duration;

use opentelemetry_sdk::{
    export::trace::SpanExporter,
    runtime::RuntimeChannel,
    trace::{BatchConfigBuilder, BatchSpanProcessor, Builder, IdGenerator, Sampler, TracerProvider},
    Resource,
};

use crate::{
    config::{TracingBatchExportConfig, TracingConfig},
    error::TracingError,
};

/// Creates a new OTEL tracing provider.
pub fn create<R, I>(
    config: TracingConfig,
    id_generator: I,
    runtime: R,
    resource: Resource,
) -> Result<TracerProvider, TracingError>
where
    R: RuntimeChannel,
    I: IdGenerator + 'static,
{
    let builder = opentelemetry_sdk::trace::TracerProvider::builder().with_config(
        opentelemetry_sdk::trace::config()
            .with_id_generator(id_generator)
            .with_sampler(Sampler::TraceIdRatioBased(config.sampling))
            .with_max_events_per_span(config.collect.max_events_per_span as u32)
            .with_max_attributes_per_span(config.collect.max_attributes_per_span as u32)
            .with_max_events_per_span(config.collect.max_events_per_span as u32)
            .with_resource(resource),
    );

    Ok(setup_exporters(builder, config, runtime)?.build())
}

fn setup_exporters<R>(
    mut tracer_provider_builder: Builder,
    config: TracingConfig,
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
    if let Some(otlp_exporter_config) = config.exporters.otlp {
        use opentelemetry_otlp::SpanExporterBuilder;
        let span_exporter = super::exporter::build_otlp_exporter::<SpanExporterBuilder>(&otlp_exporter_config)?
            .build_span_exporter()
            .map_err(|err| TracingError::SpanExporterSetup(err.to_string()))?;

        let span_processor = build_batched_span_processor(
            otlp_exporter_config.timeout,
            &otlp_exporter_config.batch_export,
            span_exporter,
            runtime.clone(),
        );
        tracer_provider_builder = tracer_provider_builder.with_span_processor(span_processor);
    }

    #[cfg(feature = "otlp")]
    if let Some(otlp_exporter_config) = config.exporters.grafbase {
        use opentelemetry_otlp::SpanExporterBuilder;
        let span_exporter = super::exporter::build_otlp_exporter::<SpanExporterBuilder>(&otlp_exporter_config)?
            .build_span_exporter()
            .map_err(|err| TracingError::SpanExporterSetup(err.to_string()))?;

        let span_processor = build_batched_span_processor(
            otlp_exporter_config.timeout,
            &otlp_exporter_config.batch_export,
            span_exporter,
            runtime.clone(),
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
