use std::collections::HashMap;
use std::net::IpAddr;
use std::num::NonZeroU32;

use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use governor::Quota;
use http::{HeaderName, HeaderValue};
use serde_json::Value;

use config::latest::{Config, GLOBAL_RATE_LIMITER};
use runtime::rate_limiting::{Error, RateLimiter, RateLimiterContext};

pub struct EngineRateLimitContext<'a>(pub &'a str);

impl RateLimiterContext for EngineRateLimitContext<'_> {
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
        Some(self.0)
    }
}

#[derive(Default)]
pub struct InMemoryRateLimiter {
    inner: HashMap<String, governor::DefaultKeyedRateLimiter<usize>>,
}

impl InMemoryRateLimiter {
    pub fn runtime(config: &Config) -> RateLimiter {
        let mut limiter = Self::default();

        // add global rate limiting configuration
        if let Some(global_rate_limit_config) = &config.rate_limit {
            limiter = limiter.with_rate_limiter(GLOBAL_RATE_LIMITER, global_rate_limit_config.clone());
        }

        // add subgraph rate limiting configuration
        for subgraph_config in config.subgraph_configs.values() {
            if let Some(rate_limit_config) = &subgraph_config.rate_limit {
                limiter = limiter.with_rate_limiter(
                    &config.strings[subgraph_config.name.0],
                    rate_limit_config.clone(),
                );
            }
        }

        RateLimiter::new(limiter)
    }

    pub fn with_rate_limiter(mut self, key: &str, rate_limit_config: config::latest::RateLimitConfig) -> Self {
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
