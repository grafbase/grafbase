use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::runtime::RuntimeChannel;
use opentelemetry_sdk::trace::IdGenerator;
use opentelemetry_sdk::Resource;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::filter::Filtered;
use tracing_subscriber::layer::Filter;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use crate::config::TelemetryConfig;
use crate::error::TracingError;

/// A type erased layer
pub type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;
/// A type erased layer filter
pub type BoxedFilter<S> = Box<dyn Filter<S> + Send + Sync + 'static>;
/// Wrapper type for a filter layer over erased layer and filter
pub type FilteredLayer<S> = Filtered<BoxedLayer<S>, BoxedFilter<S>, S>;

pub struct OtelTelemetry<Subscriber> {
    pub tracer: Option<Tracer<Subscriber>>,
    pub meter_provider: Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
    pub logger: Option<Logger>,
}

pub struct Tracer<Subscriber> {
    pub layer: OpenTelemetryLayer<Subscriber, opentelemetry_sdk::trace::Tracer>,
    pub provider: opentelemetry_sdk::trace::TracerProvider,
}

pub struct Logger {
    pub layer: OpenTelemetryTracingBridge<opentelemetry_sdk::logs::LoggerProvider, opentelemetry_sdk::logs::Logger>,
    pub provider: opentelemetry_sdk::logs::LoggerProvider,
}

/// Creates a new OTEL tracing layer that doesn't collect or export any tracing data.
/// The main reason this exists is to act as a placeholder in the subscriber. It's wrapped in a [`reload::Layer`]
/// enabling its replacement.
pub fn new_noop<S>() -> OtelTelemetry<S>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    OtelTelemetry {
        tracer: None,
        meter_provider: None,
        logger: None,
    }
}

/// Creates a new OTEL tracing layer that uses a [`BatchSpanProcessor`] to collect and export traces.
/// It's wrapped in a [`reload::Layer`] enabling its replacement.
pub fn build<S, R, I>(config: &TelemetryConfig, id_generator: I, runtime: R) -> Result<OtelTelemetry<S>, TracingError>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
    R: RuntimeChannel,
    I: IdGenerator + 'static,
{
    let mut resource_attributes: Vec<_> = config
        .resource_attributes
        .iter()
        .map(|(key, value)| KeyValue::new(key.clone(), value.clone()))
        .collect();

    resource_attributes.push(KeyValue::new("service.name", config.service_name.clone()));
    let resource = Resource::new(resource_attributes);

    let meter_provider = Some(super::metrics::build_meter_provider(
        runtime.clone(),
        config,
        resource.clone(),
    )?);

    let logger = match super::logs::build_logs_provider(runtime.clone(), config, resource.clone())? {
        Some(provider) if config.logs_exporters_enabled() => Some(Logger {
            layer: OpenTelemetryTracingBridge::new(&provider),
            provider,
        }),
        _ => None,
    };

    let tracer = if config.tracing_exporters_enabled() {
        let provider = super::traces::build_trace_provider(runtime, id_generator, config, resource.clone())?;
        let layer = tracing_opentelemetry::layer().with_tracer(
            provider
                .tracer_builder(crate::SCOPE)
                .with_version(crate::SCOPE_VERSION)
                .build(),
        );

        Some(Tracer { layer, provider })
    } else {
        None
    };

    Ok(OtelTelemetry {
        tracer,
        meter_provider,
        logger,
    })
}
