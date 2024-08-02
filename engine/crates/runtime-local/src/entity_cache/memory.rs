use std::time::Instant;

use futures_util::future::BoxFuture;

pub struct InMemoryEntityCache {
    inner: mini_moka::sync::Cache<String, CacheValue>,
}

#[derive(Clone)]
struct CacheValue {
    data: Vec<u8>,
    expires_at: Instant,
}

impl InMemoryEntityCache {
    pub fn new() -> Self {
        InMemoryEntityCache {
            inner: mini_moka::sync::Cache::new(4096),
        }
    }

    async fn get(&self, name: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let Some(value) = self.inner.get(&name.to_string()) else {
            return Ok(None);
        };

        if value.expires_at < Instant::now() {
            self.inner.invalidate(&name.to_string());
            return Ok(None);
        }

        Ok(Some(value.data))
    }

    async fn put(
        &self,
        name: &str,
        bytes: std::borrow::Cow<'_, [u8]>,
        expiration_ttl: std::time::Duration,
    ) -> anyhow::Result<()> {
        self.inner.insert(
            name.to_string(),
            CacheValue {
                data: bytes.into_owned(),
                expires_at: Instant::now() + expiration_ttl,
            },
        );
        Ok(())
    }
}

impl Default for InMemoryEntityCache {
    fn default() -> Self {
        Self::new()
    }
}

impl runtime::entity_cache::EntityCache for InMemoryEntityCache {
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
