use opentelemetry_sdk::metrics::{Aggregation, Instrument, InstrumentKind, PeriodicReader, SdkMeterProvider, Stream};
use opentelemetry_sdk::metrics::{Temporality, View};
use opentelemetry_sdk::runtime::Runtime;
use opentelemetry_sdk::Resource;
use std::time::Duration;

use crate::config::TelemetryConfig;
use crate::error::TracingError;

pub struct AggForLatencyHistogram;

impl View for AggForLatencyHistogram {
    fn match_inst(&self, inst: &Instrument) -> Option<Stream> {
        inst.kind.as_ref().map(|kind| {
            let stream = Stream::new()
                .name(inst.name.clone())
                .description(inst.description.clone())
                .unit(inst.unit.clone());

            match kind {
                InstrumentKind::Counter
                | InstrumentKind::UpDownCounter
                | InstrumentKind::ObservableCounter
                | InstrumentKind::ObservableUpDownCounter => stream.aggregation(Aggregation::Sum),
                InstrumentKind::Gauge | InstrumentKind::ObservableGauge => stream.aggregation(Aggregation::LastValue),
                InstrumentKind::Histogram => stream.aggregation(Aggregation::Base2ExponentialHistogram {
                    max_size: 160,
                    max_scale: 20,
                    record_min_max: false,
                }),
            }
        })
    }
}

pub(super) fn build_meter_provider<R>(
    runtime: R,
    config: &TelemetryConfig,
    resource: Resource,
) -> Result<SdkMeterProvider, TracingError>
where
    R: Runtime,
{
    let mut provider = SdkMeterProvider::builder()
        .with_resource(resource)
        .with_view(AggForLatencyHistogram);

    if let Some(config) = config.metrics_stdout_config() {
        let reader = PeriodicReader::builder(
            opentelemetry_stdout::MetricExporter::builder()
                .with_temporality(Temporality::Delta)
                .build(),
            runtime.clone(),
        )
        .with_interval(
            config
                .batch_export
                .unwrap_or_default()
                .scheduled_delay
                .to_std()
                .unwrap_or(Duration::from_secs(10)),
        )
        .with_timeout(config.timeout.to_std().unwrap_or(Duration::from_secs(60)))
        .build();

        provider = provider.with_reader(reader);
    }

    #[cfg(feature = "otlp")]
    if let Some(config) = config.metrics_otlp_config() {
        provider = attach_reader(config, &runtime, provider)?;
    }

    #[cfg(feature = "otlp")]
    if let Some(config) = config.grafbase_otlp_config() {
        provider = attach_reader(config, &runtime, provider)?;
    }

    Ok(provider.build())
}

#[cfg(feature = "otlp")]
fn attach_reader<R>(
    config: &crate::config::OtlpExporterConfig,
    runtime: &R,
    provider: opentelemetry_sdk::metrics::MeterProviderBuilder,
) -> Result<opentelemetry_sdk::metrics::MeterProviderBuilder, TracingError>
where
    R: Runtime,
{
    use gateway_config::OtlpExporterProtocol;
    use opentelemetry_otlp::{MetricExporter, WithExportConfig, WithHttpConfig, WithTonicConfig};

    use crate::otel::exporter::{build_metadata, build_tls_config};

    let exporter_timeout = Duration::from_secs(config.timeout.num_seconds() as u64);

    let exporter = match config.protocol {
        OtlpExporterProtocol::Grpc => {
            let grpc_config = config.grpc.clone().unwrap_or_default();

            MetricExporter::builder()
                .with_tonic()
                .with_endpoint(config.endpoint.to_string())
                .with_timeout(exporter_timeout)
                .with_metadata(build_metadata(grpc_config.headers))
                .with_tls_config(build_tls_config(grpc_config.tls)?)
                .with_temporality(Temporality::Delta)
                .build()
                .map_err(|e| TracingError::MetricsExporterSetup(e.to_string()))?
        }
        OtlpExporterProtocol::Http => {
            let http_config = config.http.clone().unwrap_or_default();

            MetricExporter::builder()
                .with_http()
                .with_endpoint(config.endpoint.to_string())
                .with_headers(http_config.headers.into_map())
                .with_timeout(exporter_timeout)
                .with_temporality(Temporality::Delta)
                .build()
                .map_err(|e| TracingError::MetricsExporterSetup(e.to_string()))?
        }
    };

    let reader = PeriodicReader::builder(exporter, runtime.clone())
        .with_interval(
            config
                .batch_export
                .scheduled_delay
                .to_std()
                .unwrap_or(Duration::from_secs(10)),
        )
        .with_timeout(config.timeout.to_std().unwrap_or(Duration::from_secs(60)))
        .build();

    Ok(provider.with_reader(reader))
}
