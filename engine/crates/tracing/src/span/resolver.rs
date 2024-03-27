use tracing::{info_span, Span};

use crate::span::ResolverInvocationRecorderSpanExt;

/// Resolver span name
pub const RESOLVER_SPAN_NAME: &str = "resolver";

/// A span for a resolver invocation
pub struct ResolverInvocationSpan<'a> {
    name: &'a str,
    error: Option<&'a str>,
    is_error: bool,
}
impl<'a> ResolverInvocationSpan<'a> {
    /// Create a new instance
    pub fn new(name: &'a str) -> Self {
        ResolverInvocationSpan {
            name,
            error: None,
            is_error: false,
        }
    }

    /// Consume self and turn into a [Span]
    pub fn into_span(self) -> Span {
        info_span!(
            target: crate::span::GRAFBASE_TARGET,
            RESOLVER_SPAN_NAME,
            "resolver.name" = self.name,
            "resolver.invocation.error" = self.error.as_ref(),
            "resolver.invocation.is_error" = self.is_error,
        )
    }
}

impl ResolverInvocationRecorderSpanExt for Span {
    fn record_failure(&self, error: &str) {
        self.record("resolver.invocation.error", error);
        self.record("resolver.invocation.is_error", true);
    }
}
