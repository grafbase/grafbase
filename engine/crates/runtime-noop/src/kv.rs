/// FIXME: Only here because of jwt-verifier, to be refactored with auth
use std::time::Duration;

use runtime::kv::{KvError, KvManager, KvResult, KvStore, KvStoreInner};

#[derive(thiserror::Error, Debug)]
pub enum NoopError {
    #[error("dummy")]
    KvNotAvailable,
}

pub struct NoopKvManager;

impl KvManager for NoopKvManager {
    fn load(&self, _namespace: &str) -> KvResult<KvStore> {
        Ok(KvStore::new(NoopKvStore))
    }
}

pub struct NoopKvStore;

#[async_trait::async_trait]
impl KvStoreInner for NoopKvStore {
    async fn get(&self, _name: &str, _cache_ttl: Option<Duration>) -> KvResult<Option<Vec<u8>>> {
        Err(KvError::Kv("Not available".into()))
    }

    async fn put(&self, _name: &str, _bytes: Vec<u8>, _expiration_ttl: Option<Duration>) -> KvResult<()> {
        Ok(())
    }
}
