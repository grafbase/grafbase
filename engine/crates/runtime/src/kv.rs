use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;

pub trait KvManager {
    type Error: std::fmt::Debug;
    type Kv: KvStore<Error = Self::Error>;

    fn load(&self, namespace: &str) -> Result<Self::Kv, Self::Error>;
}

pub trait KvStore {
    type Error: std::fmt::Debug;
    type Get: KvGet<Error = Self::Error>;
    type Put: KvPut<Error = Self::Error>;

    fn get(&self, name: &str) -> Self::Get;
    fn put<T: Serialize>(&self, name: &str, value: T) -> Result<Self::Put, Self::Error>;
}

#[async_trait::async_trait]
pub trait KvGet {
    type Error;

    #[must_use]
    fn cache_ttl(self, cache_ttl: Duration) -> Self;
    async fn json<T: DeserializeOwned>(self) -> Result<Option<T>, Self::Error>;
}

#[async_trait::async_trait]
pub trait KvPut {
    type Error;

    #[must_use]
    fn expiration_ttl(self, expiration_ttl: Duration) -> Self;
    async fn execute(self) -> Result<(), Self::Error>;
}
