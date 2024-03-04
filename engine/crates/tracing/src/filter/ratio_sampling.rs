use rand::Rng;
use tracing::span::Attributes;
use tracing::{Id, Metadata, Subscriber};
use tracing_subscriber::layer::{Context, Filter};
use tracing_subscriber::registry::{LookupSpan, SpanRef};

use crate::config::DEFAULT_SAMPLING;

fn get_random_value() -> f64 {
    rand::thread_rng().gen_range(0.0..=1.0)
}

pub struct RatioSamplingFilter(f64);

impl RatioSamplingFilter {
    pub fn new(ratio: f64) -> Self {
        Self(ratio)
    }
}

impl Default for RatioSamplingFilter {
    fn default() -> Self {
        RatioSamplingFilter::new(DEFAULT_SAMPLING)
    }
}

impl<S> Filter<S> for RatioSamplingFilter
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    /// Sampling is done at root level
    /// - if the current span has an extension marker of `Sampled`, it means that the whole chain is accepted
    /// - if the span name is equal to the expected root its subject to sampling
    /// - in any other case, we don't accept spans
    fn enabled(&self, meta: &Metadata<'_>, cx: &Context<'_, S>) -> bool {
        let current = cx.current_span();
        if let Some(span_ref) = current
            // the current span, which is the parent of the span that might get enabled here,
            // exists, but it might have been enabled by another layer like metrics
            .id()
            .and_then(|id| cx.span(id))
        {
            return span_ref.sampled();
        }

        if meta.name() != crate::span::request::SPAN_NAME {
            return false;
        }

        get_random_value() <= self.0
    }

    fn on_new_span(&self, _attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();
        if extensions.get_mut::<SampledSpan>().is_none() {
            extensions.insert(SampledSpan);
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();
        extensions.remove::<SampledSpan>();
    }
}

// courtesy from Apollo
struct SampledSpan;
pub(crate) trait Sampled {
    fn sampled(&self) -> bool;
}

impl<'a, T> Sampled for SpanRef<'a, T>
where
    T: LookupSpan<'a>,
{
    fn sampled(&self) -> bool {
        // if this extension is set, that means the parent span was accepted, and so the
        // entire trace is accepted
        let extensions = self.extensions();
        extensions.get::<SampledSpan>().is_some()
    }
}
