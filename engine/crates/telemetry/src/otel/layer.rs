use opentelemetry::trace::noop::NoopTracer;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::{Logger, LoggerProvider};
use opentelemetry_sdk::runtime::RuntimeChannel;
use opentelemetry_sdk::trace::IdGenerator;
use opentelemetry_sdk::Resource;
use tracing::Subscriber;
use tracing_subscriber::filter::Filtered;
use tracing_subscriber::layer::Filter;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{reload, Layer};

use crate::config::TelemetryConfig;
use crate::error::TracingError;

/// A type erased layer
pub type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;
/// A type erased layer filter
pub type BoxedFilter<S> = Box<dyn Filter<S> + Send + Sync + 'static>;
/// Wrapper type for a filter layer over erased layer and filter
pub type FilteredLayer<S> = Filtered<BoxedLayer<S>, BoxedFilter<S>, S>;

/// Holds tracing reloadable layer components
pub struct ReloadableOtelLayers<S> {
    /// A reloadable tracing layer
    pub tracer: Option<ReloadableOtelLayer<S, opentelemetry_sdk::trace::TracerProvider>>,
    /// A reloadable metrics layer
    pub meter_provider: Option<opentelemetry_sdk::metrics::SdkMeterProvider>,
    /// A reloadable logging layer
    pub logger: Option<OpenTelemetryTracingBridge<LoggerProvider, Logger>>,
}

/// Holds tracing reloadable layer components
pub struct ReloadableOtelLayer<Subscriber, Provider> {
    /// A reloadable layer
    pub layer: reload::Layer<BoxedLayer<Subscriber>, Subscriber>,
    /// A reloadable handle to reload a tracing layer
    pub layer_reload_handle: reload::Handle<BoxedLayer<Subscriber>, Subscriber>,
    /// The tracer provider used for tracers attached to the layer
    pub provider: Provider,
}

/// Creates a new OTEL tracing layer that doesn't collect or export any tracing data.
/// The main reason this exists is to act as a placeholder in the subscriber. It's wrapped in a [`reload::Layer`]
/// enabling its replacement.
pub fn new_noop<S>() -> ReloadableOtelLayers<S>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    ReloadableOtelLayers {
        tracer: None,
        meter_provider: None,
        logger: None,
    }
}

/// Creates a new OTEL tracing layer that uses a [`BatchSpanProcessor`] to collect and export traces.
/// It's wrapped in a [`reload::Layer`] enabling its replacement.
pub fn new_batched<S, R, I>(
    mut config: TelemetryConfig,
    id_generator: I,
    runtime: R,
    will_reload_otel: bool,
) -> Result<ReloadableOtelLayers<S>, TracingError>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
    R: RuntimeChannel,
    I: IdGenerator + 'static,
{
    let mut resource_attributes: Vec<_> = std::mem::take(&mut config.resource_attributes)
        .into_iter()
        .map(|(key, value)| KeyValue::new(key, value))
        .collect();

    resource_attributes.push(KeyValue::new("service.name", config.service_name.clone()));
    let resource = Resource::new(resource_attributes);

    // HACK: We don't want to create a PeriodicReader if we'll drop it later. Somehow it started spamming
    // stderr with:
    // 'OpenTelemetry metrics error occurred. Metrics error: reader is not registered'
    // as soon as we started waiting on the OTEL reload to be done for engine metrics.
    // So now I'm just avoiding creating it in the first place.
    let meter_provider = if will_reload_otel {
        None
    } else {
        Some(super::metrics::build_meter_provider(
            runtime.clone(),
            &config,
            resource.clone(),
        )?)
    };

    let logger = match super::logs::build_logs_provider(runtime.clone(), &config, resource.clone())? {
        Some(provider) if config.logs_exporters_enabled() => Some(OpenTelemetryTracingBridge::new(&provider)),
        _ => None,
    };

    let tracing_layer = if config.tracing_exporters_enabled() {
        let tracer_provider = super::traces::build_trace_provider(runtime, id_generator, &config, resource.clone())?;

        let tracer = tracer_provider
            .tracer_builder(crate::SCOPE)
            .with_version(crate::SCOPE_VERSION)
            .build();

        let tracer_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        let tracer_layer = tracer_layer.boxed();

        let (tracer_layer, tracer_layer_reload_handle) = reload::Layer::new(tracer_layer);

        ReloadableOtelLayer {
            layer: tracer_layer,
            layer_reload_handle: tracer_layer_reload_handle,
            provider: tracer_provider,
        }
    } else {
        let otel_layer = tracing_opentelemetry::layer().with_tracer(NoopTracer::new()).boxed();
        let (otel_layer, reload_handle) = reload::Layer::new(otel_layer);

        ReloadableOtelLayer {
            layer: otel_layer,
            layer_reload_handle: reload_handle,
            provider: Default::default(),
        }
    };

    Ok(ReloadableOtelLayers {
        tracer: Some(tracing_layer),
        meter_provider,
        logger,
    })
}
