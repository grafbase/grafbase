use crate::{config::TelemetryConfig, error::TracingError};
use opentelemetry_sdk::{logs::LoggerProvider, runtime::RuntimeChannel, Resource};

#[allow(unused_variables)]
pub(super) fn build_logs_provider<R>(
    runtime: R,
    config: &TelemetryConfig,
    resource: Resource,
) -> Result<Option<LoggerProvider>, TracingError>
where
    R: RuntimeChannel,
{
    cfg_if::cfg_if! {
        if #[cfg(feature = "otlp")] {
            use opentelemetry_sdk::logs::{BatchConfigBuilder, BatchLogProcessor};
            use std::time::Duration;

            let mut builder = LoggerProvider::builder().with_resource(resource);

            if let Some(config) = config.logs_otlp_config() {
                use opentelemetry_otlp::LogExporterBuilder;

                let exporter = match super::exporter::build_otlp_exporter(config)? {
                    either::Either::Left(grpc) => LogExporterBuilder::Tonic(grpc)
                        .build_log_exporter()
                        .map_err(|e| TracingError::LogsExporterSetup(e.to_string()))?,
                    either::Either::Right(http) => LogExporterBuilder::Http(http)
                        .build_log_exporter()
                        .map_err(|e| TracingError::LogsExporterSetup(e.to_string()))?,
                };

                let processor = {
                    let config = config.batch_export;

                    let config = BatchConfigBuilder::default()
                        .with_max_queue_size(config.max_queue_size)
                        .with_scheduled_delay(Duration::from_secs(config.scheduled_delay.num_seconds() as u64))
                        .with_max_export_batch_size(config.max_export_batch_size)
                        .build();

                    BatchLogProcessor::builder(exporter, runtime.clone())
                        .with_batch_config(config)
                        .build()
                };

                builder = builder.with_log_processor(processor);
            }

            Ok(Some(builder.build()))
        } else {
            Ok(None)
        }
    }
}
