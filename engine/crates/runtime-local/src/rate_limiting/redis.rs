mod pool;

use core::fmt;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    time::{Duration, SystemTime},
};

use anyhow::Context;
use deadpool::managed::Pool;
use futures_util::future::BoxFuture;
use grafbase_telemetry::span::GRAFBASE_TARGET;
use redis::ClientTlsConfig;
use runtime::rate_limiting::{Error, GraphRateLimit, RateLimiter, RateLimiterContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateLimitRedisConfig<'a> {
    pub url: &'a str,
    pub key_prefix: &'a str,
    pub tls: Option<RateLimitRedisTlsConfig<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateLimitRedisTlsConfig<'a> {
    pub cert: Option<&'a Path>,
    pub key: Option<&'a Path>,
    pub ca: Option<&'a Path>,
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
    pool: Pool<pool::Manager>,
    key_prefix: String,
    subgraph_limits: HashMap<String, GraphRateLimit>,
}

enum Key<'a> {
    Graph { name: &'a str },
}

impl<'a> fmt::Display for Key<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("rate_limit:")?;

        match self {
            Key::Graph { name } => {
                f.write_str(name)?;
            }
        }

        Ok(())
    }
}

impl RedisRateLimiter {
    pub async fn runtime(
        config: RateLimitRedisConfig<'_>,
        subgraph_limits: impl IntoIterator<Item = (&str, GraphRateLimit)>,
    ) -> anyhow::Result<RateLimiter> {
        let inner = Self::new(config, subgraph_limits).await?;
        Ok(RateLimiter::new(inner))
    }

    pub async fn new(
        config: RateLimitRedisConfig<'_>,
        subgraph_limits: impl IntoIterator<Item = (&str, GraphRateLimit)>,
    ) -> anyhow::Result<RedisRateLimiter> {
        let tls_config = match config.tls {
            Some(tls) => {
                let client_tls = match tls.cert.zip(tls.key) {
                    Some((cert, key)) => {
                        let mut client_cert = Vec::new();

                        File::open(cert)
                            .and_then(|file| BufReader::new(file).read_to_end(&mut client_cert))
                            .context("loading the Redis client certificate")?;

                        let mut client_key = Vec::new();

                        File::open(key)
                            .and_then(|file| BufReader::new(file).read_to_end(&mut client_key))
                            .context("loading the Redis client key")?;

                        Some(ClientTlsConfig {
                            client_cert,
                            client_key,
                        })
                    }
                    None => None,
                };

                let root_cert = match tls.ca {
                    Some(path) => {
                        let mut ca = Vec::new();

                        File::open(path)
                            .and_then(|file| BufReader::new(file).read_to_end(&mut ca))
                            .context("loading the Redis CA certificate")?;

                        Some(ca)
                    }
                    None => None,
                };

                Some(pool::TlsConfig { client_tls, root_cert })
            }
            None => None,
        };

        let manager = match pool::Manager::new(config.url, tls_config) {
            Ok(manager) => manager,
            Err(e) => {
                tracing::error!(target: GRAFBASE_TARGET, "error creating a Redis pool: {e}");
                return Err(e.into());
            }
        };

        let pool = match Pool::builder(manager)
            .wait_timeout(Some(Duration::from_secs(5)))
            .create_timeout(Some(Duration::from_secs(10)))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
        {
            Ok(pool) => pool,
            Err(e) => {
                tracing::error!(target: GRAFBASE_TARGET, "error creating a Redis pool: {e}");
                return Err(e.into());
            }
        };

        let subgraph_limits = subgraph_limits
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect();

        Ok(Self {
            pool,
            key_prefix: config.key_prefix.to_string(),
            subgraph_limits,
        })
    }

    fn generate_key(&self, bucket: u64, context: &dyn RateLimiterContext, key: Key<'_>) -> String {
        if context.is_global() {
            format!("{}:{key}:{bucket}", self.key_prefix)
        } else {
            format!("{}:subgraph:{key}:{bucket}", self.key_prefix)
        }
    }

    async fn limit_inner(&self, context: &dyn RateLimiterContext) -> Result<(), Error> {
        let Some(key) = context.key() else { return Ok(()) };

        let Some(config) = self.subgraph_limits.get(key) else {
            return Ok(());
        };

        let now = SystemTime::now();

        let current_ts = match now.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(ts) => ts.as_nanos() as u64,
            Err(error) => {
                tracing::error!(target: GRAFBASE_TARGET, "error with rate limit duration: {error}");
                return Err(Error::Internal(String::from("rate limit")));
            }
        };

        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(error) => {
                tracing::error!(target: GRAFBASE_TARGET, "error fetching a Redis connection: {error}");
                return Err(Error::Internal(String::from("rate limit")));
            }
        };

        let duration_ns = config.duration.as_nanos() as u64;
        let current_bucket = current_ts - current_ts % duration_ns;
        let previous_bucket = current_bucket - duration_ns;

        let bucket_percentage = (current_ts % duration_ns) as f64 / duration_ns as f64;

        // The counter key for the current window.
        let current_bucket = self.generate_key(current_bucket, context, Key::Graph { name: key });
        // The counter key for the previous window.
        let previous_bucket = self.generate_key(previous_bucket, context, Key::Graph { name: key });

        // We execute multiple commands in one pipelined query to be _fast_.
        let mut pipe = redis::pipe();

        // Marks the start of an atomic block. The get and incr are guaranteed to run atomically.
        pipe.atomic();

        pipe.cmd("GET").arg(&previous_bucket);
        pipe.cmd("GET").arg(&current_bucket);

        // Execute the whole pipeline in one multiplexed request.
        match pipe.query_async::<_, (Option<u64>, Option<u64>)>(&mut *conn).await {
            Ok((previous_count, current_count)) => {
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
                tracing::error!(target: GRAFBASE_TARGET, "error with Redis query: {e}");
                Err(Error::Internal(String::from("rate limit")))
            }
        }
    }
}

async fn incr_counter(pool: Pool<pool::Manager>, current_bucket: String, expire: Duration) -> Result<(), Error> {
    let mut conn = match pool.get().await {
        Ok(conn) => conn,
        Err(error) => {
            tracing::error!(target: GRAFBASE_TARGET, "error fetching a Redis connection: {error}");
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

    if let Err(e) = pipe.query_async::<_, (u64,)>(&mut *conn).await {
        tracing::error!(target: GRAFBASE_TARGET, "error with Redis query: {e}");
        return Err(Error::Internal(String::from("rate limit")));
    }

    Ok(())
}

impl runtime::rate_limiting::RateLimiterInner for RedisRateLimiter {
    fn limit<'a>(&'a self, context: &'a dyn RateLimiterContext) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(self.limit_inner(context))
    }
}
