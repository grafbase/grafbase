use std::time::{Duration, SystemTime};

use futures_util::future::BoxFuture;
use gateway_config::{Config, GraphRateLimit};
use grafbase_telemetry::otel::opentelemetry::{
    metrics::{Histogram, Meter},
    KeyValue,
};
use runtime::rate_limiting::{Error, RateLimitKey, RateLimiter, RateLimiterContext};
use tokio::sync::watch;
use tracing::{field::Empty, Instrument};

use crate::redis::Pool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateLimitRedisConfig<'a> {
    pub key_prefix: &'a str,
}

/// Rate limiter by utilizing Redis as a backend. It uses a averaging fixed window algorithm
/// to define is the limit reached or not.
///
/// The algorithm is implemented as two Redis keys: one for the current time window and another
/// for previous. The total count of the requests is the number in the current window added with
/// a percentage of the requests in the previous window. This gives us a good enough estimation,
/// around 0.003% of requests wrongly allowed or rate limited, and an average of 6% between real
/// rate and the approximate rate.
///
/// The reason for this algorithm is that it can be done without locks and with one roundtrip to
/// redis. This gives us the fastest throughput and latency.
///
/// A request must have a unique access to a connection, which means utilizing a connection
/// pool.
pub struct RedisRateLimiter {
    pool: Pool,
    key_prefix: String,
    config_watcher: watch::Receiver<Config>,
    latencies: Histogram<u64>,
}

#[derive(Debug, Clone, Copy)]
enum RedisStatus {
    Success,
    Error,
}

impl RedisStatus {
    fn as_str(self) -> &'static str {
        match self {
            RedisStatus::Success => "SUCCESS",
            RedisStatus::Error => "ERROR",
        }
    }
}

impl RedisRateLimiter {
    pub async fn runtime(
        config: RateLimitRedisConfig<'_>,
        pool: Pool,
        watcher: watch::Receiver<Config>,
        meter: &Meter,
    ) -> anyhow::Result<RateLimiter> {
        let inner = Self::new(config, pool, watcher, meter).await?;
        Ok(RateLimiter::new(inner))
    }

    async fn new(
        config: RateLimitRedisConfig<'_>,
        pool: Pool,
        watcher: watch::Receiver<Config>,
        meter: &Meter,
    ) -> anyhow::Result<RedisRateLimiter> {
        Ok(Self {
            pool,
            key_prefix: config.key_prefix.to_string(),
            config_watcher: watcher,
            latencies: meter.u64_histogram("grafbase.gateway.rate_limit.duration").init(),
        })
    }

    fn generate_key(&self, bucket: u64, key: &RateLimitKey<'_>) -> String {
        match key {
            RateLimitKey::Global => {
                format!("{}:rate_limit:global:{bucket}", self.key_prefix)
            }
            RateLimitKey::Subgraph(ref graph) => {
                format!("{}:subgraph:rate_limit:{graph}:{bucket}", self.key_prefix)
            }
        }
    }

    fn record_duration(&self, duration: Duration, status: RedisStatus) {
        let attributes = vec![KeyValue::new("grafbase.redis.status", status.as_str())];
        self.latencies.record(duration.as_millis() as u64, &attributes);
    }

    async fn limit_inner(&self, key: &RateLimitKey<'_>, config: GraphRateLimit) -> Result<(), Error> {
        let now = SystemTime::now();

        let current_ts = match now.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(ts) => ts.as_nanos() as u64,
            Err(error) => {
                tracing::error!("error with rate limit duration: {error}");
                return Err(Error::Internal(String::from("rate limit")));
            }
        };

        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(error) => {
                tracing::error!("error fetching a Redis connection: {error}");
                return Err(Error::Internal(String::from("rate limit")));
            }
        };

        let duration_ns = config.duration.as_nanos() as u64;
        let current_bucket = current_ts - current_ts % duration_ns;
        let previous_bucket = current_bucket - duration_ns;

        let bucket_percentage = (current_ts % duration_ns) as f64 / duration_ns as f64;

        // The counter key for the current window.
        let current_bucket = self.generate_key(current_bucket, key);
        // The counter key for the previous window.
        let previous_bucket = self.generate_key(previous_bucket, key);

        // We execute multiple commands in one pipelined query to be _fast_.
        let mut pipe = redis::pipe();

        // Marks the start of an atomic block. The get and incr are guaranteed to run atomically.
        pipe.atomic();

        pipe.cmd("GET").arg(&previous_bucket);
        pipe.cmd("GET").arg(&current_bucket);

        let start = SystemTime::now();
        let result = pipe.query_async::<(Option<u64>, Option<u64>)>(&mut *conn).await;
        let duration = SystemTime::now().duration_since(start).unwrap_or_default();

        // Execute the whole pipeline in one multiplexed request.
        match result {
            Ok((previous_count, current_count)) => {
                self.record_duration(duration, RedisStatus::Success);

                let previous_count = previous_count.unwrap_or_default().min(config.limit as u64);
                let current_count = current_count.unwrap_or_default().min(config.limit as u64);

                // Sum is a percentage what is left from the previous window, and the count of the
                // current window.
                let average = previous_count as f64 * (1.0 - bucket_percentage) + current_count as f64;

                if average < config.limit as f64 {
                    tokio::spawn(incr_counter(self.pool.clone(), current_bucket, config.duration));

                    Ok(())
                } else {
                    Err(Error::ExceededCapacity)
                }
            }
            Err(e) => {
                self.record_duration(duration, RedisStatus::Error);
                tracing::error!("error with Redis query: {e}");
                Err(Error::Internal(String::from("rate limit")))
            }
        }
    }
}

async fn incr_counter(pool: Pool, current_bucket: String, expire: Duration) -> Result<(), Error> {
    let mut conn = match pool.get().await {
        Ok(conn) => conn,
        Err(error) => {
            tracing::error!("error fetching a Redis connection: {error}");
            return Err(Error::Internal(String::from("rate limit")));
        }
    };

    let mut pipe = redis::pipe();
    pipe.atomic();

    pipe.cmd("INCR").arg(&current_bucket);

    // Sets the timeout to the set. This will delete the data after the duration if we do not modify the value.
    pipe.cmd("EXPIRE")
        .arg(&current_bucket)
        .arg(expire.as_secs() * 2)
        .ignore();

    if let Err(e) = pipe.query_async::<(u64,)>(&mut *conn).await {
        tracing::error!("error with Redis query: {e}");
        return Err(Error::Internal(String::from("rate limit")));
    }

    Ok(())
}

impl runtime::rate_limiting::RateLimiterInner for RedisRateLimiter {
    fn limit<'a>(&'a self, context: &'a dyn RateLimiterContext) -> BoxFuture<'a, Result<(), Error>> {
        let Some(key) = context.key() else {
            return Box::pin(async { Ok(()) });
        };

        let config = match key {
            RateLimitKey::Global => self
                .config_watcher
                .borrow()
                .gateway
                .rate_limit
                .as_ref()
                .and_then(|rt| rt.global),
            RateLimitKey::Subgraph(name) => self
                .config_watcher
                .borrow()
                .subgraphs
                .get(name.as_ref())
                .and_then(|sb| sb.rate_limit),
        };

        let Some(config) = config else {
            return Box::pin(async { Ok(()) });
        };

        let span = tracing::info_span!("rate limit", "subgraph.name" = Empty);

        if let RateLimitKey::Subgraph(subgraph) = key {
            span.record("subgraph.name", subgraph.as_ref());
        }

        Box::pin(self.limit_inner(key, config).instrument(span))
    }
}
