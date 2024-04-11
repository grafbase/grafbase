use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_sdk::runtime::RuntimeChannel;
use opentelemetry_sdk::trace::IdGenerator;
use opentelemetry_sdk::Resource;
use tracing::Subscriber;
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::filter::Filtered;
use tracing_subscriber::layer::Filter;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{reload, Layer};

use crate::config::TracingConfig;
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
    pub meter: Option<ReloadableOtelLayer<S, opentelemetry_sdk::metrics::SdkMeterProvider>>,
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
        meter: None,
    }
}

/// Creates a new OTEL tracing layer that uses a [`BatchSpanProcessor`] to collect and export traces.
/// It's wrapped in a [`reload::Layer`] enabling its replacement.
pub fn new_batched<S, R, I>(
    service_name: impl Into<String>,
    config: TracingConfig,
    id_generator: I,
    runtime: R,
    resource_attributes: impl Into<Vec<KeyValue>>,
) -> Result<ReloadableOtelLayers<S>, TracingError>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
    R: RuntimeChannel,
    I: IdGenerator + 'static,
{
    let mut resource_attributes = resource_attributes.into();
    resource_attributes.push(KeyValue::new("service.name", service_name.into()));
    let resource = Resource::new(resource_attributes);

    let meter_provider = super::metrics::build_meter_provider(runtime.clone(), &config, resource.clone())?;
    let meter_layer = MetricsLayer::<S>::new(meter_provider.clone()).boxed();
    let (meter_layer, meter_layer_reload_handle) = reload::Layer::new(meter_layer);

    let tracer_provider = super::traces::create(config, id_generator, runtime, resource.clone())?;
    let tracer = tracer_provider.tracer("batched-otel");
    let tracer_layer = tracing_opentelemetry::layer().with_tracer(tracer).boxed();
    let (tracer_layer, tracer_layer_reload_handle) = reload::Layer::new(tracer_layer);

    Ok(ReloadableOtelLayers {
        tracer: Some(ReloadableOtelLayer {
            layer: tracer_layer,
            layer_reload_handle: tracer_layer_reload_handle,
            provider: tracer_provider,
        }),
        meter: Some(ReloadableOtelLayer {
            layer: meter_layer,
            layer_reload_handle: meter_layer_reload_handle,
            provider: meter_provider,
        }),
    })
}
