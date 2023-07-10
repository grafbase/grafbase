mod edge;
mod error;
mod gcache;
mod global;
mod key;

use async_trait::async_trait;
use dynaql::parser::types::OperationType;
pub use edge::EdgeCache;
pub use error::CacheError;
pub use gcache::{Cache, CacheResponse};
#[cfg(any(feature = "local", feature = "sqlite"))]
pub use global::noop::NoopGlobalCache;
#[cfg(all(not(feature = "local"), not(feature = "sqlite")))]
pub use global::remote::CloudflareGlobal;
pub use key::CacheKey;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;

pub type CacheResult<T> = Result<T, CacheError>;

pub enum CacheReadStatus {
    Hit,
    Bypass,
    Miss { max_age: Duration },
    Stale { is_updating: bool },
}

#[derive(Debug, Default, Eq, PartialEq, Hash, strum::Display, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum CacheEntryState {
    Fresh,
    #[default]
    Stale,
    Updating,
}

impl ToString for CacheReadStatus {
    fn to_string(&self) -> String {
        match self {
            CacheReadStatus::Hit => "HIT".to_string(),
            CacheReadStatus::Miss { .. } => "MISS".to_string(),
            CacheReadStatus::Stale { is_updating } => {
                if *is_updating {
                    "UPDATING".to_string()
                } else {
                    "STALE".to_string()
                }
            }
            CacheReadStatus::Bypass => "BYPASS".to_string(),
        }
    }
}

pub enum CacheProviderResponse<Type> {
    Hit(Type),
    Miss,
    Stale { response: Type, is_updating: bool },
}

#[async_trait::async_trait(?Send)]
pub trait CacheProvider {
    type Value;

    async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheProviderResponse<Self::Value>> {
        unimplemented!()
    }

    async fn put(
        _cache_name: &str,
        _ray_id: &str,
        _key: &str,
        _status: CacheEntryState,
        _value: Arc<Self::Value>,
        _tags: Vec<String>,
    ) -> CacheResult<()> {
        unimplemented!()
    }

    async fn delete(_cache_name: &str, _ray_id: &str, _key: &str) -> CacheResult<()> {
        unimplemented!()
    }
}

pub trait Cacheable: DeserializeOwned + Serialize {
    fn max_age_seconds(&self) -> usize;
    fn stale_seconds(&self) -> usize;
    fn ttl_seconds(&self) -> usize;
    fn cache_tags(&self, priority_tags: Vec<String>) -> Vec<String>;
    fn should_purge_related(&self) -> bool;
    fn should_cache(&self) -> bool;
}

impl Cacheable for dynaql::Response {
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

// this trait exists because:
// - the cache api from workers is specific to colocations
// - the cache api from workers doesn't support delete by tags
// - its not meant to be statically dispatched
#[async_trait(?Send)]
pub trait GlobalCacheProvider {
    async fn purge_by_tags(&self, _tags: Vec<String>) -> CacheResult<()> {
        unimplemented!()
    }
    async fn purge_by_hostname(&self, _hostname: String) -> CacheResult<()> {
        unimplemented!()
    }
}
