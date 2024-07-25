use runtime::kv::{KvResult, KvStore, KvStoreInner};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Mutex,
    time::{Duration, Instant},
};

pub struct InMemoryKvStore {
    inner: Mutex<HashMap<String, CacheValue>>,
}

struct CacheValue {
    data: Vec<u8>,
    expires_at: Option<Instant>,
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
    async fn get(&self, name: &str, _cache_ttl: Option<Duration>) -> KvResult<Option<Vec<u8>>> {
        let mut lock = self.inner.lock().unwrap();
        let Entry::Occupied(entry) = lock.entry(name.to_string()) else {
            return Ok(None);
        };

        let value = entry.get();

        match value.expires_at {
            Some(instant) if instant < Instant::now() => {
                entry.remove();
                Ok(None)
            }
            _ => Ok(Some(value.data.clone())),
        }
    }

    #[allow(clippy::panic)]
    async fn put(&self, name: &str, bytes: Vec<u8>, expiration_ttl: Option<Duration>) -> KvResult<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.insert(
            name.to_string(),
            CacheValue {
                data: bytes,
                expires_at: expiration_ttl.map(|ttl| Instant::now() + ttl),
            },
        );
        // Sanity check, we're never deleting anything currently. And only used store OpenID
        // providers metadata. Easier to deal with a panic than a memory leak.
        if inner.len() > 1000 {
            panic!("Too many entries in in-memory kv store");
        }
        Ok(())
    }
}
