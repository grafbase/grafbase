use std::borrow::Cow;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::future::BoxFuture;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Too many requests")]
    ExceededCapacity,
    #[error("internal error: {0}")]
    Internal(String),
}

pub trait RateLimiterContext: Send + Sync {
    fn header(&self, name: http::HeaderName) -> Option<&http::HeaderValue>;
    fn graphql_operation_name(&self) -> Option<&str>;
    fn ip(&self) -> Option<IpAddr>;
    fn jwt_claim(&self, key: &str) -> Option<&serde_json::Value>;

    fn key(&self) -> Option<&RateLimitKey<'_>> {
        None
    }
}

pub trait RateLimiterInner: Send + Sync {
    fn limit<'a>(&'a self, context: &'a dyn RateLimiterContext) -> BoxFuture<'a, Result<(), Error>>;
}

#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<dyn RateLimiterInner>,
}

impl RateLimiter {
    pub fn new(rate_limiter: impl RateLimiterInner + 'static) -> RateLimiter {
        RateLimiter {
            inner: Arc::new(rate_limiter),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RateLimitKey<'a> {
    Global,
    Subgraph(Cow<'a, str>),
}

impl<'a> From<&'a str> for RateLimitKey<'a> {
    fn from(value: &'a str) -> Self {
        Self::Subgraph(Cow::Borrowed(value))
    }
}

impl<'a> From<String> for RateLimitKey<'a> {
    fn from(value: String) -> Self {
        Self::Subgraph(Cow::Owned(value))
    }
}

impl<'a> RateLimiterContext for RateLimitKey<'a> {
    fn header(&self, _: http::HeaderName) -> Option<&http::HeaderValue> {
        None
    }

    fn graphql_operation_name(&self) -> Option<&str> {
        None
    }

    fn ip(&self) -> Option<IpAddr> {
        None
    }

    fn jwt_claim(&self, _: &str) -> Option<&serde_json::Value> {
        None
    }

    fn key(&self) -> Option<&RateLimitKey<'a>> {
        Some(self)
    }
}

impl std::ops::Deref for RateLimiter {
    type Target = dyn RateLimiterInner;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphRateLimit {
    pub limit: usize,
    pub duration: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct KeyedRateLimitConfig {
    pub rate_limiting_configs: HashMap<String, GraphRateLimit>,
}
