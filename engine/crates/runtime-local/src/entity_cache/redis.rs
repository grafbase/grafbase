use deadpool::managed::Object;
use futures_util::future::BoxFuture;
use grafbase_telemetry::span::GRAFBASE_TARGET;
use redis::{AsyncCommands, SetOptions};

use crate::redis::{Manager, Pool};

pub struct RedisEntityCache {
    pool: Pool,
    key_prefix: String,
}

impl RedisEntityCache {
    pub fn new(pool: Pool, key_prefix: &str) -> Self {
        RedisEntityCache {
            pool,
            key_prefix: key_prefix.to_string(),
        }
    }

    async fn get(&self, name: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let mut connection = self.connection().await?;
        Ok(connection.get(self.key(name)).await?)
    }

    async fn put(
        &self,
        name: &str,
        bytes: std::borrow::Cow<'_, [u8]>,
        expiration_ttl: std::time::Duration,
    ) -> anyhow::Result<()> {
        let mut connection = self.connection().await?;
        let options = SetOptions::default().with_expiration(self.expiry_time(expiration_ttl));
        Ok(connection.set_options(self.key(name), bytes.as_ref(), options).await?)
    }

    fn key(&self, name: &str) -> String {
        format!("{}-{name}", self.key_prefix)
    }

    fn expiry_time(&self, duration: std::time::Duration) -> redis::SetExpiry {
        if duration.as_secs() > 60 {
            redis::SetExpiry::PX(duration.as_millis() as usize)
        } else {
            redis::SetExpiry::EX(duration.as_secs() as usize)
        }
    }

    async fn connection(&self) -> Result<Object<Manager>, anyhow::Error> {
        match self.pool.get().await {
            Ok(conn) => Ok(conn),
            Err(error) => {
                tracing::error!(target: GRAFBASE_TARGET, "error fetching a Redis connection: {error}");
                anyhow::bail!("error fetching a redis connection: {error}");
            }
        }
    }
}

impl runtime::entity_cache::EntityCache for RedisEntityCache {
    fn get<'a>(&'a self, name: &'a str) -> BoxFuture<'a, anyhow::Result<Option<Vec<u8>>>> {
        Box::pin(self.get(name))
    }

    fn put<'a>(
        &'a self,
        name: &'a str,
        bytes: std::borrow::Cow<'a, [u8]>,
        expiration_ttl: std::time::Duration,
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(self.put(name, bytes, expiration_ttl))
    }
}
