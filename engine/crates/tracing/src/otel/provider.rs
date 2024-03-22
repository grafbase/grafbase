use std::time::Duration;

use opentelemetry::KeyValue;
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
    service_name: impl Into<String>,
    config: TracingConfig,
    id_generator: I,
    runtime: R,
) -> Result<TracerProvider, TracingError>
where
    R: RuntimeChannel,
    I: IdGenerator + 'static,
{
    let service_name = service_name.into();

    let builder = opentelemetry_sdk::trace::TracerProvider::builder().with_config(
        opentelemetry_sdk::trace::config()
            .with_id_generator(id_generator)
            .with_sampler(Sampler::TraceIdRatioBased(config.sampling))
            .with_max_events_per_span(config.collect.max_events_per_span as u32)
            .with_max_attributes_per_span(config.collect.max_attributes_per_span as u32)
            .with_max_events_per_span(config.collect.max_events_per_span as u32)
            .with_resource(Resource::new(vec![KeyValue::new("service.name", service_name)])),
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
        use opentelemetry_otlp::{SpanExporterBuilder, WithExportConfig};
        use std::str::FromStr;
        use tonic::metadata::MetadataKey;
        use tonic::transport::ClientTlsConfig;

        use crate::config::TracingOtlpExporterProtocol;

        let span_exporter = {
            let exporter_timeout = Duration::from_secs(otlp_exporter_config.timeout.num_seconds() as u64);

            match otlp_exporter_config.protocol {
                TracingOtlpExporterProtocol::Grpc => {
                    let grpc_config = otlp_exporter_config.grpc.unwrap_or_default();

                    let metadata = {
                        // note: I'm not using MetadataMap::from_headers due to `http` crate version issues.
                        // we're using 1 but otel currently pins tonic to an older version that requires 0.2.
                        // once versions get aligned we can replace the following
                        let headers = grpc_config.headers.try_into_map()?;

                        let metadata = tonic::metadata::MetadataMap::with_capacity(headers.len());

                        headers
                            .into_iter()
                            .fold(metadata, |mut acc, (header_name, header_value)| {
                                let key = MetadataKey::from_str(&header_name).unwrap();
                                acc.insert(key, header_value.parse().unwrap());
                                acc
                            })
                    };

                    let mut grpc_exporter = opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint(otlp_exporter_config.endpoint.to_string())
                        .with_timeout(exporter_timeout)
                        .with_metadata(metadata);

                    if let Some(tls_config) = grpc_config.tls {
                        grpc_exporter = grpc_exporter.with_tls_config(ClientTlsConfig::try_from(tls_config)?);
                    }

                    SpanExporterBuilder::from(grpc_exporter)
                        .build_span_exporter()
                        .map_err(|err| TracingError::SpanExporterSetup(err.to_string()))?
                }
                TracingOtlpExporterProtocol::Http => {
                    let http_config = otlp_exporter_config.http.unwrap_or_default();

                    let http_exporter = opentelemetry_otlp::new_exporter()
                        .http()
                        .with_endpoint(otlp_exporter_config.endpoint.to_string())
                        .with_timeout(exporter_timeout)
                        .with_headers(http_config.headers.try_into_map()?);

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
