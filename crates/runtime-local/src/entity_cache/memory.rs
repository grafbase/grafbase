use std::time::Instant;

use bytes::Bytes;
use futures_util::{FutureExt, future::BoxFuture};
use tracing::{Instrument, field::Empty};

pub struct InMemoryEntityCache {
    inner: mini_moka::sync::Cache<String, CacheValue>,
}

#[derive(Clone)]
struct CacheValue {
    data: Bytes,
    expires_at: Instant,
}

impl InMemoryEntityCache {
    pub fn new() -> Self {
        InMemoryEntityCache {
            inner: mini_moka::sync::Cache::new(4096),
        }
    }

    async fn get(&self, name: &str) -> anyhow::Result<Option<Bytes>> {
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
                data: bytes.into_owned().into(),
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
    fn get<'a>(&'a self, name: &'a str) -> BoxFuture<'a, anyhow::Result<Option<Bytes>>> {
        let cache_span = tracing::info_span!(
            "entity cache get",
            "grafbase.entity_cache.status" = Empty,
            "otel.status_code" = Empty,
        );

        let cache_get = self
            .get(name)
            .instrument(cache_span.clone())
            .inspect(move |item| match item {
                Ok(Some(_)) => {
                    cache_span.record("grafbase.entity_cache.status", "HIT");
                }
                Ok(None) => {
                    cache_span.record("grafbase.entity_cache.status", "MISS");
                }
                Err(e) => {
                    cache_span.record("otel.status_code", "Error");
                    cache_span.record("grafbase.entity_cache.error", e.to_string());
                }
            });

        Box::pin(cache_get)
    }

    fn put<'a>(
        &'a self,
        name: &'a str,
        bytes: std::borrow::Cow<'a, [u8]>,
        expiration_ttl: std::time::Duration,
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        let cache_span = tracing::info_span!("entity cache put");
        Box::pin(self.put(name, bytes, expiration_ttl).instrument(cache_span))
    }
}
