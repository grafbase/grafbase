use engine::parser::types::OperationType;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

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

pub enum Entry<T> {
    Hit(T),
    Miss,
    Stale {
        response: T,
        state: EntryState,
        is_early_stale: bool,
    },
}

#[async_trait::async_trait(?Send)]
pub trait Cache {
    type Value: Cacheable + 'static;

    async fn get(&self, namespace: &str, key: &str) -> Result<Entry<Self::Value>>;

    async fn put(
        &self,
        namespace: &str,
        ray_id: &str,
        key: &str,
        state: EntryState,
        value: Arc<Self::Value>,
        tags: Vec<String>,
    ) -> Result<()>;

    async fn delete(&self, namespace: &str, ray_id: &str, key: &str) -> Result<()>;

    async fn purge_by_tags(&self, tags: Vec<String>) -> Result<()>;
    async fn purge_by_hostname(&self, hostname: String) -> Result<()>;
}

pub trait Cacheable: DeserializeOwned + Serialize {
    fn max_age_seconds(&self) -> usize;
    fn stale_seconds(&self) -> usize;
    fn ttl_seconds(&self) -> usize;
    fn cache_tags(&self, priority_tags: Vec<String>) -> Vec<String>;
    fn should_purge_related(&self) -> bool;
    fn should_cache(&self) -> bool;
}

impl Cacheable for engine::Response {
    fn max_age_seconds(&self) -> usize {
        self.cache_control.max_age
    }

    fn stale_seconds(&self) -> usize {
        self.cache_control.stale_while_revalidate
    }

    fn ttl_seconds(&self) -> usize {
        self.cache_control.max_age + self.cache_control.stale_while_revalidate
    }

    fn cache_tags(&self, mut priority_tags: Vec<String>) -> Vec<String> {
        let response_tags = self.data.cache_tags().iter().cloned().collect::<Vec<_>>();
        priority_tags.extend(response_tags);

        priority_tags
    }

    fn should_purge_related(&self) -> bool {
        self.operation_type == OperationType::Mutation && !self.data.cache_tags().is_empty()
    }

    fn should_cache(&self) -> bool {
        self.operation_type != OperationType::Mutation && self.errors.is_empty() && self.cache_control.max_age != 0
    }
}

#[cfg(feature = "test-utils")]
pub mod test_utils {
    use super::*;

    #[async_trait::async_trait(?Send)]
    pub trait FakeCache {
        type Value: Cacheable + 'static;

        async fn get(&self, _namespace: &str, _key: &str) -> Result<Entry<Self::Value>> {
            unimplemented!()
        }

        async fn put(
            &self,
            _namespace: &str,
            _ray_id: &str,
            _key: &str,
            _status: EntryState,
            _value: Arc<Self::Value>,
            _tags: Vec<String>,
        ) -> Result<()> {
            unimplemented!()
        }

        async fn delete(&self, _namespace: &str, _ray_id: &str, _key: &str) -> Result<()> {
            unimplemented!()
        }

        async fn purge_by_tags(&self, _tags: Vec<String>) -> Result<()> {
            unimplemented!()
        }

        async fn purge_by_hostname(&self, _hostname: String) -> Result<()> {
            unimplemented!()
        }
    }

    #[async_trait::async_trait(?Send)]
    impl<T: FakeCache> Cache for T {
        type Value = <T as FakeCache>::Value;

        async fn get(&self, namespace: &str, key: &str) -> Result<Entry<Self::Value>> {
            self.get(namespace, key).await
        }

        async fn put(
            &self,
            namespace: &str,
            ray_id: &str,
            key: &str,
            status: EntryState,
            value: Arc<Self::Value>,
            tags: Vec<String>,
        ) -> Result<()> {
            self.put(namespace, ray_id, key, status, value, tags).await
        }

        async fn delete(&self, namespace: &str, ray_id: &str, key: &str) -> Result<()> {
            self.delete(namespace, ray_id, key).await
        }

        async fn purge_by_tags(&self, tags: Vec<String>) -> Result<()> {
            self.purge_by_tags(tags).await
        }

        async fn purge_by_hostname(&self, hostname: String) -> Result<()> {
            self.purge_by_hostname(hostname).await
        }
    }
}
