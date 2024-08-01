use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Mutex,
    time::Instant,
};

use futures_util::future::BoxFuture;

#[derive(Default)]
pub struct InMemoryEntityCache {
    inner: Mutex<HashMap<String, CacheValue>>,
}

struct CacheValue {
    data: Vec<u8>,
    expires_at: Instant,
}

impl InMemoryEntityCache {
    async fn get(&self, name: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let mut lock = self.inner.lock().unwrap();
        let Entry::Occupied(entry) = lock.entry(name.to_string()) else {
            return Ok(None);
        };

        let value = entry.get();

        if value.expires_at < Instant::now() {
            entry.remove();
            return Ok(None);
        }

        Ok(Some(value.data.clone()))
    }

    async fn put(
        &self,
        name: &str,
        bytes: std::borrow::Cow<'_, [u8]>,
        expiration_ttl: std::time::Duration,
    ) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.insert(
            name.to_string(),
            CacheValue {
                data: bytes.into_owned(),
                expires_at: Instant::now() + expiration_ttl,
            },
        );
        Ok(())
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
