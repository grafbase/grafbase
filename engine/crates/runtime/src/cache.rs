use serde::{de::DeserializeOwned, Serialize};
use std::{sync::Arc, time::Duration};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    CacheGet(String),
    #[error("{0}")]
    CachePut(String),
    #[error("{0}")]
    CacheDelete(String),
    #[error("{0}")]
    CachePurgeByTags(String),
    #[error("Origin error: {0}")]
    Origin(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, strum::Display, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum EntryState {
    Fresh,
    #[default]
    Stale,
    UpdateInProgress,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Entry<T> {
    Hit(T),
    Miss,
    Stale {
        response: T,
        state: EntryState,
        is_early_stale: bool,
    },
}

#[async_trait::async_trait]
pub trait Cache: Send + Sync {
    type Value: Cacheable + 'static;

    async fn get(&self, key: &str) -> Result<Entry<Self::Value>>;
    async fn put(&self, key: &str, state: EntryState, value: Arc<Self::Value>, tags: Vec<String>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn purge_by_tags(&self, tags: Vec<String>) -> Result<()>;
    async fn purge_by_hostname(&self, hostname: String) -> Result<()>;
}

pub trait Cacheable: DeserializeOwned + Serialize + Send + Sync {
    // Also retrieved during cache.get(), so needs to be included in the value.
    fn max_age(&self) -> Duration;
    fn stale_while_revalidate(&self) -> Duration;
    fn cache_tags(&self) -> Vec<String>;
    fn should_purge_related(&self) -> bool;
    fn should_cache(&self) -> bool;

    fn cache_tags_with_priority_tags(&self, mut tags: Vec<String>) -> Vec<String> {
        tags.extend(self.cache_tags());
        tags
    }
}

#[cfg(feature = "test-utils")]
pub mod test_utils {
    use super::*;

    #[async_trait::async_trait]
    pub trait FakeCache: Send + Sync {
        type Value: Cacheable + 'static;

        async fn get(&self, _key: &str) -> Result<Entry<Self::Value>> {
            unimplemented!()
        }

        async fn put(
            &self,
            _key: &str,
            _status: EntryState,
            _value: Arc<Self::Value>,
            _tags: Vec<String>,
        ) -> Result<()> {
            unimplemented!()
        }

        async fn delete(&self, _key: &str) -> Result<()> {
            unimplemented!()
        }

        async fn purge_by_tags(&self, _tags: Vec<String>) -> Result<()> {
            unimplemented!()
        }

        async fn purge_by_hostname(&self, _hostname: String) -> Result<()> {
            unimplemented!()
        }
    }

    #[async_trait::async_trait]
    impl<T: FakeCache> Cache for T {
        type Value = <T as FakeCache>::Value;

        async fn get(&self, key: &str) -> Result<Entry<Self::Value>> {
            self.get(key).await
        }

        async fn put(&self, key: &str, status: EntryState, value: Arc<Self::Value>, tags: Vec<String>) -> Result<()> {
            self.put(key, status, value, tags).await
        }

        async fn delete(&self, key: &str) -> Result<()> {
            self.delete(key).await
        }

        async fn purge_by_tags(&self, tags: Vec<String>) -> Result<()> {
            self.purge_by_tags(tags).await
        }

        async fn purge_by_hostname(&self, hostname: String) -> Result<()> {
            self.purge_by_hostname(hostname).await
        }
    }
}
