use crate::span::CacheRecorderSpanExt;
use http::HeaderValue;
use tracing::{info_span, Span};

/// Cache span name
pub const CACHE_SPAN_NAME: &str = "cache";

/// A span that captures a potentially cached operation
pub struct CacheSpan {
    status: HeaderValue,
    is_error: Option<bool>,
}
impl CacheSpan {
    /// Create a new instance
    pub fn new(status: HeaderValue) -> Self {
        CacheSpan { status, is_error: None }
    }

    /// Consume self and turn into a [Span]
    pub fn into_span(self) -> Span {
        info_span!(
            target: crate::span::GRAFBASE_TARGET,
            CACHE_SPAN_NAME,
            "cache.status" = self.status.to_str().ok(),
            "cache.is_error" = self.is_error,
        )
    }
}

impl CacheRecorderSpanExt for Span {
    fn record_status(&self, value: HeaderValue) {
        self.record("cache.status", value.to_str().ok());
    }

    fn record_error(&self) {
        self.record("cache.is_error", true);
    }
}
