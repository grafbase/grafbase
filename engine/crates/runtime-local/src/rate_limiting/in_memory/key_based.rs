use std::num::NonZeroU32;
use std::sync::Arc;
use std::{collections::HashMap, sync::RwLock};

use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use governor::Quota;
use grafbase_telemetry::span::GRAFBASE_TARGET;

use runtime::rate_limiting::{Error, GraphRateLimit, RateLimitKey, RateLimiter, RateLimiterContext};
use tokio::sync::watch;

type Limits = HashMap<RateLimitKey<'static>, GraphRateLimit>;
type Limiters = HashMap<RateLimitKey<'static>, governor::DefaultKeyedRateLimiter<usize>>;

pub struct InMemoryRateLimiter {
    limiters: Arc<RwLock<Limiters>>,
}

impl InMemoryRateLimiter {
    pub fn runtime(mut updates: watch::Receiver<Limits>) -> RateLimiter {
        let mut limiters = HashMap::new();

        // add subgraph rate limiting configuration
        for (name, config) in updates.borrow_and_update().iter() {
            let Some(limiter) = create_limiter(*config) else {
                continue;
            };

            limiters.insert(name.clone(), limiter);
        }

        let limiters = Arc::new(RwLock::new(limiters));
        let limiters_copy = Arc::downgrade(&limiters);

        tokio::spawn(async move {
            while let Ok(()) = updates.changed().await {
                let Some(limiters) = limiters_copy.upgrade() else {
                    break;
                };

                let mut limiters = limiters.write().unwrap();
                limiters.clear();

                for (name, config) in updates.borrow_and_update().iter() {
                    let Some(limiter) = create_limiter(*config) else {
                        continue;
                    };

                    limiters.insert(name.clone(), limiter);
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
