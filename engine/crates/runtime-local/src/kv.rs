use runtime::kv::{KvResult, KvStore, KvStoreInner};
use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

pub struct InMemoryKvStore {
    inner: Mutex<HashMap<String, (Vec<u8>, Instant)>>,
}

impl InMemoryKvStore {
    pub fn runtime() -> KvStore {
        KvStore::new(Self::default())
    }
}

impl Default for InMemoryKvStore {
    fn default() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl KvStoreInner for InMemoryKvStore {
    async fn get(&self, name: &str, cache_ttl: Option<Duration>) -> KvResult<Option<Vec<u8>>> {
        if let Some(value) = self.inner.lock().unwrap().get(name) {
            if let Some(cache_ttl) = cache_ttl {
                if value.1.elapsed() > cache_ttl {
                    return Ok(None);
                }
            }
            Ok(Some(value.0.clone()))
        } else {
            Ok(None)
        }
    }

    #[allow(clippy::panic)]
    async fn put(&self, name: &str, bytes: Vec<u8>, expiration_ttl: Option<Duration>) -> KvResult<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.insert(
            name.to_string(),
            (
                bytes,
                expiration_ttl.map(|ttl| Instant::now() + ttl).unwrap_or(Instant::now()),
            ),
        );
        // Sanity check, we're never deleting anything currently. And only used store OpenID
        // providers metadata. Easier to deal with a panic than a memory leak.
        if inner.len() > 1000 {
            panic!("Too many entries in in-memory kv store");
        }
        Ok(())
    }
}
