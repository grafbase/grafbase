mod pool;

use core::fmt;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    net::IpAddr,
    path::Path,
    time::{Duration, SystemTime},
};

use anyhow::Context;
use deadpool::managed::Pool;
use futures_util::future::BoxFuture;
use grafbase_telemetry::span::GRAFBASE_TARGET;
use http::{HeaderName, HeaderValue};
use runtime::rate_limiting::{Error, GraphRateLimit, RateLimiter, RateLimiterContext};
use serde_json::Value;

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

/// Rate limiter by utilizing Redis as a backend. It uses a sliding window algorithm
/// to define is the limit reached or not.
///
/// The sliding window is implemented as a Redis sorted set. For every request, we add
/// its timestamp in nanoseconds to the set, and move the window forward by dropping the
/// items outside of the window. This is done with a Redis pipeline, packing all commands
/// into a single request to save time. The number of requests in the window is calculated
/// by taking a count of the ordered set;.
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
                f.write_str(":all")?;
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
        let manager = match config.tls {
            Some(tls) => {
                let mut client_cert = Vec::new();

                File::open(tls.cert)
                    .and_then(|file| BufReader::new(file).read_to_end(&mut client_cert))
                    .context("loading the Redis client certificate")?;

                let mut client_key = Vec::new();

                File::open(tls.key)
                    .and_then(|file| BufReader::new(file).read_to_end(&mut client_key))
                    .context("loading the Redis client key")?;

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

                pool::Manager::new(
                    config.url,
                    Some(pool::TlsConfig {
                        client_cert,
                        client_key,
                        root_cert,
                    }),
                )
                .context("creating a Redis connection with TLS")?
            }
            None => pool::Manager::new(config.url, None).context("initializing a Redis client")?,
        };

        let pool = Pool::builder(manager)
            .wait_timeout(Some(Duration::from_secs(5)))
            .create_timeout(Some(Duration::from_secs(10)))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .context("creating a Redis connection pool")?;

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

    fn generate_key(&self, key: Key<'_>) -> String {
        format!("{}:{key}", self.key_prefix)
    }

    async fn limit_inner(&self, context: &dyn RateLimiterContext) -> Result<(), Error> {
        let Some(key) = context.key() else { return Ok(()) };

        let Some(config) = self.subgraph_limits.get(key) else {
            return Ok(());
        };

        let now = SystemTime::now();

        let current_ts = match now.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(ts) => ts.as_nanos(),
            Err(e) => return Err(Error::Internal(e.to_string())),
        };

        let window_start_ts = match now
            .checked_sub(config.duration)
            .and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok())
        {
            Some(ts) => ts.as_nanos(),
            None => {
                tracing::error!(target: GRAFBASE_TARGET, "{key} duration was set to too high");
                return Err(Error::Internal(String::from("invalid rate limit duration")));
            }
        };

        let key = self.generate_key(Key::Graph { name: key });

        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(error) => {
                tracing::error!(target: GRAFBASE_TARGET, "error fetching a Redis connection: {error}");
                return Err(Error::Internal(error.to_string()));
            }
        };

        // Implements a Redis transaction but with async code. Copying what they do in their sync transaction helper:
        // https://docs.rs/redis/latest/src/redis/connection.rs.html#1520-1547
        //
        // This is not safe if we share the same connection with other requests. That's why we utilize deadpool
        // in the limitter, which guarantees we get a unique access to the connection. See details in the issue:
        // https://github.com/redis-rs/redis-rs/issues/1257
        //
        // The transaction returns null on a race situation, which means we immediately retry until a value comes out.
        loop {
            // Lock the key from others:
            // https://redis.io/docs/latest/commands/watch/
            redis::cmd("WATCH")
                .arg(&[&key])
                .query_async::<_, ()>(&mut *conn)
                .await
                .map_err(|e| Error::Internal(e.to_string()))?;

            // We execute multiple commands in one pipelined query to be _fast_.
            let mut pipe = redis::pipe();

            // Marks the start of a transaction block. Subsequent commands will be queued for atomic execution using EXEC.
            pipe.atomic();

            // Removes all elements in the sorted set stored at key with a score between min and max (inclusive).
            // From forever ago until when our rate limit window starts.
            // https://redis.io/docs/latest/commands/zremrangebyscore/
            pipe.cmd("ZREMRANGEBYSCORE")
                .arg(&key)
                .arg("-inf")
                .arg(format!("{window_start_ts}"))
                .ignore();

            // Adds the timestamp twice: once as a value and again as a score to the set. (the set is sorted by the score)
            // https://redis.io/docs/latest/commands/zadd/
            pipe.cmd("ZADD")
                .arg(&key)
                .arg("NX")
                .arg(current_ts as u64)
                .arg(current_ts as u64)
                .ignore();

            // Counts how many requests we have in the set.
            // https://redis.io/docs/latest/commands/zcount/
            pipe.zcount(&key, "-inf", "+inf");

            // Sets the timeout to the set. This will delete the data if we do not modify the set.
            // https://redis.io/docs/latest/commands/expire/
            pipe.cmd("EXPIRE").arg(&key).arg(config.duration.as_secs()).ignore();

            // Execute the whole pipeline in one multiplexed request.
            let result = pipe.query_async::<_, Option<(u64,)>>(&mut *conn).await;

            match result {
                Ok(Some((count,))) => {
                    // Here we tell others this key is again available to read, unlocking other pending transactions.
                    // https://redis.io/docs/latest/commands/unwatch/
                    redis::cmd("UNWATCH")
                        .query_async::<_, ()>(&mut *conn)
                        .await
                        .map_err(|e| Error::Internal(e.to_string()))?;

                    if count <= config.limit as u64 {
                        return Ok(());
                    } else {
                        return Err(Error::ExceededCapacity);
                    }
                }
                Ok(None) => continue,
                Err(e) => {
                    tracing::error!(target: GRAFBASE_TARGET, "error with a Redis query: {e}");
                    return Err(Error::Internal(e.to_string()));
                }
            };
        }
    }
}

impl runtime::rate_limiting::RateLimiterInner for RedisRateLimiter {
    fn limit<'a>(&'a self, context: &'a dyn RateLimiterContext) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(self.limit_inner(context))
    }
}
