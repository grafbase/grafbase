mod edge;
mod error;
mod gcache;
mod key;

pub use edge::EdgeCache;
pub use error::CacheError;
pub use gcache::Cache;
pub use key::CacheKey;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;

pub type CacheResult<T> = Result<T, CacheError>;

pub enum CacheReadStatus {
    Hit,
    Miss { max_age: Duration },
    Stale { is_updating: bool },
}

#[derive(
    Debug, Default, Eq, PartialEq, Hash, strum_macros::Display, strum_macros::EnumString, strum_macros::IntoStaticStr,
)]
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
        }
    }
}

pub enum CacheResponse<Type> {
    Hit(Type),
    Miss(Type),
    Stale { response: Type, is_updating: bool },
}

#[async_trait::async_trait(?Send)]
pub trait CacheProvider {
    type Value;

    async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheResponse<Self::Value>> {
        unimplemented!()
    }

    async fn put(
        _cache_name: &str,
        _ray_id: &str,
        _key: &str,
        _status: CacheEntryState,
        _value: Arc<Self::Value>,
    ) -> CacheResult<()> {
        unimplemented!()
    }

    async fn delete(_cache_name: &str, _ray_id: &str, _key: &str) -> CacheResult<()> {
        unimplemented!()
    }
}

pub trait Cacheable: DeserializeOwned + Serialize + Default {
    fn max_age_seconds(&self) -> usize;
    fn stale_seconds(&self) -> usize;
    fn ttl_seconds(&self) -> usize;
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
}
