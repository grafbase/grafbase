use std::num::NonZeroU32;
use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};

use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use gateway_config::{Config, GraphRateLimit};
use governor::Quota;
use grafbase_telemetry::span::GRAFBASE_TARGET;

use runtime::rate_limiting::{Error, RateLimitKey, RateLimiter, RateLimiterContext};
use tokio::sync::watch;

type Limiters = HashMap<RateLimitKey<'static>, governor::DefaultKeyedRateLimiter<usize>>;

pub struct InMemoryRateLimiter {
    limiters: Arc<RwLock<Limiters>>,
}

/// Load the rate limit configuration for global and subgraph level settings.
pub fn as_keyed_rate_limit_config(config: &Config) -> HashMap<RateLimitKey<'static>, GraphRateLimit> {
    let mut key_based_config = HashMap::new();

    if let Some(global_config) = config.gateway.rate_limit.as_ref().and_then(|c| c.global) {
        key_based_config.insert(RateLimitKey::Global, global_config);
    }

    for (subgraph_name, subgraph) in config.subgraphs.iter() {
        if let Some(limit) = subgraph.rate_limit {
            key_based_config.insert(RateLimitKey::Subgraph(subgraph_name.clone().into()), limit);
        }
    }

    key_based_config
}

impl InMemoryRateLimiter {
    pub fn runtime(rate_limiting_configs: HashMap<RateLimitKey<'static>, GraphRateLimit>) -> RateLimiter {
        let mut limiters = HashMap::new();

        // add subgraph rate limiting configuration
        for (key, limits) in rate_limiting_configs {
            let Some(limiter) = create_limiter(limits) else {
                continue;
            };

            limiters.insert(key.clone(), limiter);
        }

        let limiters = Arc::new(RwLock::new(limiters));
        RateLimiter::new(Self { limiters })
    }

    pub fn runtime_with_watcher(mut config: watch::Receiver<Config>) -> RateLimiter {
        let mut limiters = HashMap::new();
        let rate_limiting_configs = as_keyed_rate_limit_config(&config.borrow());

        // add subgraph rate limiting configuration
        for (key, limits) in rate_limiting_configs {
            let Some(limiter) = create_limiter(limits) else {
                continue;
            };

            limiters.insert(key.clone(), limiter);
        }

        let limiters = Arc::new(RwLock::new(limiters));
        let limiters_copy = Arc::downgrade(&limiters);

        tokio::spawn(async move {
            while let Ok(()) = config.changed().await {
                let Some(limiters) = limiters_copy.upgrade() else {
                    break;
                };

                let mut limiters = limiters.write().unwrap();
                limiters.clear();

                let rate_limiting_configs = as_keyed_rate_limit_config(&config.borrow());
                for (key, limits) in rate_limiting_configs {
                    let Some(limiter) = create_limiter(limits) else {
                        continue;
                    };

                    limiters.insert(key, limiter);
                }
            }
        });

        RateLimiter::new(Self { limiters })
    }
}

fn create_limiter(rate_limit_config: GraphRateLimit) -> Option<governor::DefaultKeyedRateLimiter<usize>> {
    let Some(quota) = (rate_limit_config.limit as u64).checked_div(rate_limit_config.duration.as_secs()) else {
        tracing::error!(target: GRAFBASE_TARGET, "the duration for rate limit cannot be zero");
        return None;
    };

    let Some(quota) = NonZeroU32::new(quota as u32) else {
        tracing::error!(target: GRAFBASE_TARGET, "the limit is too low per defined duration");
        return None;
    };

    Some(governor::RateLimiter::keyed(Quota::per_second(quota)))
}

impl runtime::rate_limiting::RateLimiterInner for InMemoryRateLimiter {
    fn limit<'a>(&'a self, context: &'a dyn RateLimiterContext) -> BoxFuture<'a, Result<(), Error>> {
        async {
            let Some(key) = context.key() else { return Ok(()) };
            let limiters = self.limiters.read().unwrap();

            if let Some(rate_limiter) = limiters.get(key) {
                rate_limiter
                    .check_key(&usize::MIN)
                    .map_err(|_err| Error::ExceededCapacity)?;
            };

            Ok(())
        }
        .boxed()
    }
}
