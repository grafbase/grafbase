use crate::{config::TelemetryConfig, error::TracingError};

use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider};

#[allow(unused_variables)]
pub(super) fn build_logs_provider(
    config: &TelemetryConfig,
    resource: Resource,
) -> Result<Option<SdkLoggerProvider>, TracingError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "otlp")] {
            use opentelemetry_otlp::{LogExporter, WithExportConfig, WithHttpConfig, WithTonicConfig};
            use crate::otel::exporter::{build_metadata, build_tls_config};
            use opentelemetry_sdk::logs::{BatchConfigBuilder, BatchLogProcessor};
            use std::time::Duration;

            let mut builder = SdkLoggerProvider::builder().with_resource(resource);

            if let Some(config) = config.logs_otlp_config() {
                let exporter_timeout = Duration::from_secs(config.timeout.num_seconds() as u64);

                let exporter = match config.protocol {
                    gateway_config::OtlpExporterProtocol::Grpc => {
                        let grpc_config = config.grpc.clone().unwrap_or_default();

                        LogExporter::builder()
                            .with_tonic()
                            .with_endpoint(config.endpoint.to_string())
                            .with_timeout(exporter_timeout)
                            .with_metadata(build_metadata(grpc_config.headers))
                            .with_tls_config(build_tls_config(grpc_config.tls)?)
                            .build()
                            .map_err(|e| TracingError::LogsExporterSetup(e.to_string()))?
                    },
                    gateway_config::OtlpExporterProtocol::Http => {
                        let http_config = config.http.clone().unwrap_or_default();

                        LogExporter::builder()
                            .with_http()
                            .with_endpoint(config.endpoint.to_string())
                            .with_headers(http_config.headers.into_map())
                            .with_timeout(exporter_timeout)
                            .build()
                            .map_err(|e| TracingError::LogsExporterSetup(e.to_string()))?
                    },
                };

                let processor = {
                    let config = config.batch_export;

                    let config = BatchConfigBuilder::default()
                        .with_max_queue_size(config.max_queue_size)
                        .with_scheduled_delay(Duration::from_secs(config.scheduled_delay.num_seconds() as u64))
                        .with_max_export_batch_size(config.max_export_batch_size)
                        .build();

                    BatchLogProcessor::builder(exporter)
                        .with_batch_config(config)
                        .build()
                };

                builder = builder.with_log_processor(processor);
            };

            Ok(Some(builder.build()))
        } else {
            Ok(None)
        }
    }
}
