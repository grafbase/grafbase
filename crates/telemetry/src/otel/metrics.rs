use gateway_config::{LayeredOtlExporterConfig, OtlpExporterProtocolConfig};
use opentelemetry_otlp::Protocol;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{
    Aggregation, Instrument, InstrumentKind, PeriodicReader, SdkMeterProvider, Stream, Temporality, View,
};

use crate::config::TelemetryConfig;
use crate::error::TracingError;

pub struct DeltaTemporality;

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

pub(super) fn build_meter_provider(
    config: &TelemetryConfig,
    resource: Resource,
) -> Result<SdkMeterProvider, TracingError> {
    let mut provider = SdkMeterProvider::builder()
        .with_resource(resource)
        .with_view(AggForLatencyHistogram);

    if let Some(config) = config.metrics_stdout_config() {
        let reader = PeriodicReader::builder(
            opentelemetry_stdout::MetricExporter::builder()
                .with_temporality(Temporality::Delta)
                .build(),
        )
        .with_interval(config.batch_export.unwrap_or_default().scheduled_delay)
        .build();

        provider = provider.with_reader(reader);
    }

    if let Some(config) = config.metrics_otlp_config() {
        provider = attach_reader(config, provider)?;
    }

    if let Some(config) = config.grafbase_otlp_config() {
        provider = attach_reader(
            LayeredOtlExporterConfig {
                global: config.clone(),
                local: config,
            },
            provider,
        )?;
    }

    Ok(provider.build())
}

fn attach_reader(
    config: LayeredOtlExporterConfig,
    provider: opentelemetry_sdk::metrics::MeterProviderBuilder,
) -> Result<opentelemetry_sdk::metrics::MeterProviderBuilder, TracingError> {
    use opentelemetry_otlp::{MetricExporter, WithExportConfig, WithHttpConfig, WithTonicConfig};

    use crate::otel::exporter::{build_metadata, build_tls_config};

    let exporter_timeout = config.timeout();

    let exporter = match config.protocol() {
        OtlpExporterProtocolConfig::Grpc(grpc_config) => MetricExporter::builder()
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
            .with_temporality(Temporality::Delta)
            .build()
            .map_err(|e| TracingError::MetricsExporterSetup(e.to_string()))?,
        OtlpExporterProtocolConfig::Http(http_config) => MetricExporter::builder()
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
                            url.set_path("/v1/metrics");
                        }
                        url.to_string()
                    }))
                    .unwrap_or("http://127.0.0.1:4318/v1/metrics".to_string()),
            )
            .with_headers(http_config.headers.into_map())
            .with_timeout(exporter_timeout)
            .with_temporality(Temporality::Delta)
            .build()
            .map_err(|e| TracingError::MetricsExporterSetup(e.to_string()))?,
    };

    let reader = PeriodicReader::builder(exporter)
        .with_interval(config.batch_export().scheduled_delay)
        .build();

    Ok(provider.with_reader(reader))
}
