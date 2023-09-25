/// FIXME: Only here because of jwt-verifier, to be refactored with auth
use std::time::Duration;

use runtime::kv::{KvGet, KvManager, KvPut, KvStore};
use serde::de::DeserializeOwned;

#[derive(thiserror::Error, Debug)]
pub enum NoopError {
    #[error("dummy")]
    KvNotAvailable,
}

pub struct NoopKv;

impl KvManager for NoopKv {
    type Error = NoopError;
    type Kv = NoopKvStore;

    fn load(&self, namespace: &str) -> Result<Self::Kv, Self::Error> {
        let _ = namespace;
        Err(NoopError::KvNotAvailable)
    }
}

#[derive(Default)]
pub struct NoopKvStore;

impl KvStore for NoopKvStore {
    type Error = NoopError;
    type Get = NoopKvGet;
    type Put = NoopKvPut;

    fn get(&self, _name: &str) -> Self::Get {
        unimplemented!()
    }

    fn put<T: serde::Serialize>(&self, _name: &str, _value: T) -> Result<Self::Put, Self::Error> {
        unimplemented!()
    }
}

pub struct NoopKvGet;

#[async_trait::async_trait]
impl KvGet for NoopKvGet {
    type Error = NoopError;

    fn cache_ttl(self, _cache_ttl: Duration) -> Self {
        unimplemented!()
    }

    async fn json<T: DeserializeOwned>(self) -> Result<Option<T>, Self::Error> {
        unimplemented!()
    }
}

pub struct NoopKvPut;

#[async_trait::async_trait]
impl KvPut for NoopKvPut {
    type Error = NoopError;

    fn expiration_ttl(self, _expiration_ttl: Duration) -> Self {
        unimplemented!()
    }

    async fn execute(self) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
