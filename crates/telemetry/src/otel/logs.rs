/// Custom logs bridge that fixes trace context propagation
pub mod bridge;

use crate::{config::TelemetryConfig, error::TracingError};

use gateway_config::OtlpExporterProtocolConfig;
use opentelemetry_otlp::Protocol;
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider};

#[allow(unused_variables)]
pub(super) fn build_logs_provider(
    config: &TelemetryConfig,
    resource: Resource,
) -> Result<Option<SdkLoggerProvider>, TracingError> {
    use crate::otel::exporter::{build_metadata, build_tls_config};
    use opentelemetry_otlp::{LogExporter, WithExportConfig, WithHttpConfig, WithTonicConfig};
    use opentelemetry_sdk::logs::{BatchConfigBuilder, BatchLogProcessor};

    let mut builder = SdkLoggerProvider::builder().with_resource(resource);

    if let Some(config) = config.logs_otlp_config() {
        let exporter_timeout = config.timeout();

        let exporter = match config.protocol() {
            OtlpExporterProtocolConfig::Grpc(grpc_config) => LogExporter::builder()
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
                .map_err(|e| TracingError::LogsExporterSetup(e.to_string()))?,
            OtlpExporterProtocolConfig::Http(http_config) => LogExporter::builder()
                .with_http()
                .with_protocol(Protocol::HttpBinary)
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
                                url.set_path("/v1/logs");
                            }
                            url.to_string()
                        }))
                        .unwrap_or("http://127.0.0.1:4318/v1/logs".to_string()),
                )
                .with_headers(http_config.headers.into_map())
                .with_timeout(exporter_timeout)
                .build()
                .map_err(|e| TracingError::LogsExporterSetup(e.to_string()))?,
        };

        let processor = {
            let config = config.batch_export();

            let config = BatchConfigBuilder::default()
                .with_max_queue_size(config.max_queue_size)
                .with_scheduled_delay(config.scheduled_delay)
                .with_max_export_batch_size(config.max_export_batch_size)
                .build();

            BatchLogProcessor::builder(exporter).with_batch_config(config).build()
        };

        builder = builder.with_log_processor(processor);
    };

    Ok(Some(builder.build()))
}
