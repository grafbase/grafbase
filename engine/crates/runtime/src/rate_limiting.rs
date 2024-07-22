use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
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
    fn key(&self) -> Option<&str> {
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

impl std::ops::Deref for RateLimiter {
    type Target = dyn RateLimiterInner;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubgraphRateLimitConfig {
    pub limit: usize,
    pub duration: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct KeyedRateLimitConfig<'a> {
    pub rate_limiting_configs: HashMap<&'a str, SubgraphRateLimitConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateLimitRedisConfig<'a> {
    pub url: &'a str,
    pub key_prefix: &'a str,
    pub tls: Option<RateLimitRedisTlsConfig<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateLimitRedisTlsConfig<'a> {
    pub cert: &'a Path,
    pub key: &'a Path,
    pub ca: Option<&'a Path>,
}
