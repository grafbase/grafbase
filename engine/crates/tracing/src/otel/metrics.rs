use opentelemetry_sdk::metrics::data::Temporality;
use opentelemetry_sdk::metrics::reader::{AggregationSelector, TemporalitySelector};
use opentelemetry_sdk::metrics::{Aggregation, InstrumentKind, PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::runtime::Runtime;
use opentelemetry_sdk::Resource;
use std::time::Duration;

use crate::config::TracingConfig;
use crate::error::TracingError;

pub(super) fn build_meter_provider<R>(
    runtime: R,
    config: &TracingConfig,
    resource: Resource,
) -> Result<SdkMeterProvider, TracingError>
where
    R: Runtime,
{
    let mut provider = SdkMeterProvider::builder().with_resource(resource);

    if config
        .exporters
        .stdout
        .as_ref()
        .map(|cfg| cfg.enabled)
        .unwrap_or_default()
    {
        let reader = PeriodicReader::builder(opentelemetry_stdout::MetricsExporter::default(), runtime.clone())
            .with_interval(Duration::from_secs(10))
            .with_timeout(Duration::from_secs(10))
            .build();

        provider = provider.with_reader(reader);
    }

    #[cfg(feature = "otlp")]
    if let Some(config) = config.exporters.otlp.as_ref().filter(|cfg| cfg.enabled) {
        use opentelemetry_otlp::MetricsExporterBuilder;
        let exporter = super::exporter::build_otlp_exporter::<MetricsExporterBuilder>(config)?
            .build_metrics_exporter(Box::new(DeltaTemporality), Box::new(AggForLatencyHistogram))
            .map_err(|e| TracingError::MetricsExporterSetup(e.to_string()))?;
        let reader = PeriodicReader::builder(exporter, runtime.clone())
            .with_interval(Duration::from_secs(10))
            .with_timeout(Duration::from_secs(10))
            .build();

        provider = provider.with_reader(reader);
    }

    Ok(provider.build())
}

struct DeltaTemporality;

impl TemporalitySelector for DeltaTemporality {
    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        Temporality::Delta
    }
}

struct AggForLatencyHistogram;

impl AggregationSelector for AggForLatencyHistogram {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        match kind {
            InstrumentKind::Counter
            | InstrumentKind::UpDownCounter
            | InstrumentKind::ObservableCounter
            | InstrumentKind::ObservableUpDownCounter => Aggregation::Sum,
            InstrumentKind::Gauge | InstrumentKind::ObservableGauge => Aggregation::LastValue,
            // Using Java SDK defaults.
            InstrumentKind::Histogram => Aggregation::Base2ExponentialHistogram {
                max_size: 160,
                max_scale: 20,
                record_min_max: false,
            },
        }
    }
}
