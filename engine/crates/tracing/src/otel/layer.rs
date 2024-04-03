use opentelemetry::trace::noop::NoopTracer;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_sdk::runtime::RuntimeChannel;
use opentelemetry_sdk::trace::IdGenerator;
use tracing::Subscriber;
use tracing_subscriber::filter::{FilterExt, Filtered};
use tracing_subscriber::layer::Filter;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{reload, EnvFilter, Layer};

use crate::config::TracingConfig;
use crate::error::TracingError;

use super::provider;

/// A type erased layer
pub type BoxedLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;
/// A type erased layer filter
pub type BoxedFilter<S> = Box<dyn Filter<S> + Send + Sync + 'static>;
/// Wrapper type for a filter layer over erased layer and filter
pub type FilteredLayer<S> = Filtered<BoxedLayer<S>, BoxedFilter<S>, S>;

/// Holds tracing reloadable layer components
pub struct ReloadableLayer<S> {
    /// A reloadable layer
    pub layer: reload::Layer<BoxedLayer<S>, S>,
    /// A reloadable handle to reload a tracing layer
    pub handle: reload::Handle<BoxedLayer<S>, S>,
    /// The tracer provider used for tracers attached to the layer
    pub provider: Option<opentelemetry_sdk::trace::TracerProvider>,
}

/// Creates a new OTEL tracing layer that doesn't collect or export any tracing data.
/// The main reason this exists is to act as a placeholder in the subscriber. It's wrapped in a [`reload::Layer`]
/// enabling its replacement.
pub fn new_noop<S>() -> ReloadableLayer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(NoopTracer::new())
        .with_filter(FilterExt::boxed(EnvFilter::new("off")))
        .boxed();

    let (otel_layer, reload_handle) = reload::Layer::new(otel_layer);

    ReloadableLayer {
        layer: otel_layer,
        handle: reload_handle,
        provider: None,
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
) -> Result<ReloadableLayer<S>, TracingError>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
    R: RuntimeChannel,
    I: IdGenerator + 'static,
{
    let provider = provider::create(service_name, config, id_generator, runtime, resource_attributes)?;
    let tracer = provider.tracer("batched-otel");

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer).boxed();

    let (otel_layer, reload_handle) = reload::Layer::new(otel_layer);

    Ok(ReloadableLayer {
        layer: otel_layer,
        handle: reload_handle,
        provider: Some(provider),
    })
}
