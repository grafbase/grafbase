mod cached;
mod control;
mod status;

pub use cached::*;
pub use control::*;
use futures_util::future::BoxFuture;
use serde::{de::DeserializeOwned, Serialize};
pub use status::*;
use std::{
    borrow::Cow,
    future::IntoFuture,
    result::Result,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::async_runtime::AsyncRuntime;

static X_GRAFBASE_CACHE: http::HeaderName = http::HeaderName::from_static("x-grafbase-cache");
const VALUE_NAMESPACE: &str = "v";
const UPDATE_IN_PROGRESS_NAMEPACE: &str = "u";

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("cache: {0}")]
    Cache(String),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct Cache {
    raw: Arc<dyn RawCache>,
    async_runtime: AsyncRuntime,
}

pub trait RawCache: 'static + Send + Sync {
    fn get<'a>(&'a self, namespace: &'a str, key: &'a str) -> BoxFuture<'a, Result<Entry<Vec<u8>>, CacheError>>;
    fn put<'a>(
        &'a self,
        namespace: &'a str,
        key: &'a str,
        value: Cow<'a, [u8]>,
        tags: Vec<String>,
        max_age: Duration,
        max_stale: Duration,
    ) -> BoxFuture<'a, Result<(), CacheError>>;
    fn delete<'a>(&'a self, namespace: &'a str, key: &'a str) -> BoxFuture<'a, Result<(), CacheError>>;
    fn purge_by_tags(&self, tags: Vec<String>) -> BoxFuture<'_, Result<(), CacheError>>;
    fn purge_all(&self) -> BoxFuture<'_, Result<(), CacheError>>;
}

/// Wraps an entry from cache when getting it from there
#[derive(Debug, PartialEq, Eq)]
pub enum Entry<T> {
    Hit {
        value: T,
        stale_at: Instant,
        invalid_at: Instant,
    },
    Stale {
        value: T,
        invalid_at: Instant,
    },
    Miss,
}

impl<T> Entry<T> {
    fn into_value(self) -> Option<T> {
        match self {
            Entry::Hit { value, .. } => Some(value),
            Entry::Stale { value, .. } => Some(value),
            Entry::Miss => None,
        }
    }
}

impl Cache {
    pub fn new(raw: impl RawCache, async_runtime: AsyncRuntime) -> Self {
        Self {
            raw: Arc::new(raw),
            async_runtime,
        }
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError> {
        self.get(key)
            .await?
            .map(|value| serde_json::from_slice(&value).map_err(Into::into))
            .transpose()
    }

    pub fn put_json<'a, T: Serialize>(
        &'a self,
        key: &'a str,
        value: &T,
        max_age: Duration,
    ) -> CachePut<'a, Result<Cow<'a, [u8]>, CacheError>> {
        CachePut {
            cache: self,
            key,
            value: serde_json::to_vec(value).map_err(Into::into).map(Cow::Owned),
            tags: Vec::new(),
            max_age,
            max_stale: Duration::ZERO,
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError> {
        self.raw.get(VALUE_NAMESPACE, key).await.map(Entry::into_value)
    }

    pub fn put<'a>(
        &'a self,
        key: &'a str,
        value: impl Into<Cow<'a, [u8]>>,
        max_age: Duration,
    ) -> CachePut<'a, Cow<'a, [u8]>> {
        CachePut {
            cache: self,
            key,
            value: value.into(),
            tags: Vec::new(),
            max_age,
            max_stale: Duration::ZERO,
        }
    }

    pub async fn delete(&self, key: &str) -> Result<(), CacheError> {
        self.raw.delete(VALUE_NAMESPACE, key).await
    }

    pub async fn purge_by_tags(&self, tags: Vec<String>) -> Result<(), CacheError> {
        self.raw.purge_by_tags(tags).await
    }

    pub async fn purge_all(&self) -> Result<(), CacheError> {
        self.raw.purge_all().await
    }
}

pub struct CachePut<'a, V> {
    cache: &'a Cache,
    key: &'a str,
    value: V,
    tags: Vec<String>,
    max_age: Duration,
    max_stale: Duration,
}

impl<V> CachePut<'_, V> {
    pub fn with_tags(self, tags: Vec<String>) -> Self {
        Self { tags, ..self }
    }

    pub fn with_max_stale(self, max_stale: Duration) -> Self {
        Self { max_stale, ..self }
    }
}

impl<'a> IntoFuture for CachePut<'a, Cow<'a, [u8]>> {
    type Output = Result<(), CacheError>;

    type IntoFuture = BoxFuture<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let CachePut {
            cache,
            key,
            value,
            tags,
            max_age,
            max_stale,
        } = self;
        cache.raw.put(VALUE_NAMESPACE, key, value, tags, max_age, max_stale)
    }
}

impl<'a> IntoFuture for CachePut<'a, Result<Cow<'a, [u8]>, CacheError>> {
    type Output = Result<(), CacheError>;

    type IntoFuture = BoxFuture<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let CachePut {
            cache,
            key,
            value,
            tags,
            max_age,
            max_stale,
        } = self;
        Box::pin(async move {
            cache
                .raw
                .put(VALUE_NAMESPACE, key, value?, tags, max_age, max_stale)
                .await
        })
    }
}
