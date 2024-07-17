use std::collections::HashMap;
use std::net::IpAddr;
use std::num::NonZeroU32;

use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use governor::Quota;
use serde_json::Value;

use http::{HeaderName, HeaderValue};
use runtime::rate_limiting::{Error, KeyedRateLimitConfig, RateLimitConfig, RateLimiter, RateLimiterContext};

pub struct RateLimitingContext(pub String);

impl RateLimiterContext for RateLimitingContext {
    fn header(&self, _name: HeaderName) -> Option<&HeaderValue> {
        None
    }

    fn graphql_operation_name(&self) -> Option<&str> {
        None
    }

    fn ip(&self) -> Option<IpAddr> {
        None
    }

    fn jwt_claim(&self, _key: &str) -> Option<&Value> {
        None
    }

    fn key(&self) -> Option<&str> {
        Some(&self.0)
    }
}

#[derive(Default)]
pub struct InMemoryRateLimiter {
    inner: HashMap<String, governor::DefaultKeyedRateLimiter<usize>>,
}

impl InMemoryRateLimiter {
    pub fn runtime(config: KeyedRateLimitConfig<'_>) -> RateLimiter {
        let mut limiter = Self::default();

        // add subgraph rate limiting configuration
        for (name, rate_limit_config) in config.rate_limiting_configs {
            limiter = limiter.with_rate_limiter(name, rate_limit_config.clone());
        }

        RateLimiter::new(limiter)
    }

    pub fn with_rate_limiter(mut self, key: &str, rate_limit_config: RateLimitConfig) -> Self {
        let quota = (rate_limit_config.limit as u64)
            .checked_div(rate_limit_config.duration.as_secs())
            .expect("rate limiter with invalid per second quota");

        self.inner.insert(
            key.to_string(),
            governor::RateLimiter::keyed(Quota::per_second(
                NonZeroU32::new(quota as u32).expect("rate limit duration cannot be 0"),
            )),
        );
        self
    }
}

impl runtime::rate_limiting::RateLimiterInner for InMemoryRateLimiter {
    fn limit<'a>(&'a self, context: &'a dyn RateLimiterContext) -> BoxFuture<'a, Result<(), Error>> {
        async {
            if let Some(key) = context.key() {
                if let Some(rate_limiter) = self.inner.get(key) {
                    rate_limiter
                        .check_key(&usize::MIN)
                        .map_err(|_err| Error::ExceededCapacity)?;
                };
            }

            Ok(())
        }
        .boxed()
    }
}
