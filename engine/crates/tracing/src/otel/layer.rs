use opentelemetry::global;
use opentelemetry::trace::noop::NoopTracer;
use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::runtime::RuntimeChannel;
use opentelemetry_sdk::trace::RandomIdGenerator;
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

/// Creates a new OTEL tracing layer that doesn't collect or export any tracing data.
/// The main reason this exists is to act as a placeholder in the subscriber. It's wrapped in a [`reload::Layer`]
/// enabling its replacement.
// Note: this returns a `FilteredLayer` because of https://github.com/tokio-rs/tracing/issues/1629
// it could very well just return `BoxedLayer<S>` and the handler for reload would just work with `.replace()` without panicking
pub fn new_noop<S>() -> (reload::Layer<FilteredLayer<S>, S>, reload::Handle<FilteredLayer<S>, S>)
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(NoopTracer::new())
        .boxed()
        .with_filter(FilterExt::boxed(EnvFilter::new("off")));

    let (otel_layer, reload_handle) = reload::Layer::new(otel_layer);

    (otel_layer, reload_handle)
}

/// Creates a new OTEL tracing layer that uses a [`BatchSpanProcessor`] to collect and export traces
pub fn new_batched<S, R>(
    service_name: impl Into<String>,
    config: TracingConfig,
    runtime: R,
) -> Result<BoxedLayer<S>, TracingError>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
    R: RuntimeChannel,
{
    let provider = provider::create(service_name, config, RandomIdGenerator::default(), runtime)?;
    let tracer = provider.tracer("batched-otel");

    let _ = global::set_tracer_provider(provider);

    Ok(tracing_opentelemetry::layer().with_tracer(tracer).boxed())
}
