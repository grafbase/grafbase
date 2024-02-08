mod cached;

use headers::HeaderMapExt;
use std::{sync::Arc, time::Duration};

use serde::{de::DeserializeOwned, Serialize};

const X_GRAFBASE_CACHE: &str = "x-grafbase-cache";

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
/// Represents the state an entry can be inside the cache
pub enum EntryState {
    Fresh,
    #[default]
    Stale,
    UpdateInProgress,
}

/// Wraps an entry from cache when getting it from there
#[derive(Debug, PartialEq, Eq)]
pub enum Entry<T> {
    Hit(T),
    Miss,
    Stale(StaleEntry<T>),
}

impl<T> Entry<T> {
    fn try_map<V, F: FnOnce(T) -> Result<V>>(self, f: F) -> Result<Entry<V>> {
        match self {
            Entry::Hit(value) => f(value).map(Entry::Hit),
            Entry::Miss => Ok(Entry::Miss),
            Entry::Stale(entry) => f(entry.value).map(|value| {
                Entry::Stale(StaleEntry {
                    value,
                    state: entry.state,
                    is_early_stale: entry.is_early_stale,
                    metadata: entry.metadata,
                })
            }),
        }
    }

    pub fn into_value(self) -> Option<T> {
        match self {
            Entry::Hit(value) => Some(value),
            Entry::Miss => None,
            Entry::Stale(StaleEntry { value, .. }) => Some(value),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StaleEntry<T> {
    pub value: T,
    pub state: EntryState,
    pub is_early_stale: bool,
    pub metadata: CacheMetadata,
}

/// Represents the status of the cache read operation
#[derive(Debug, PartialEq, Eq)]
pub enum CacheReadStatus {
    Hit,
    Bypass,
    Miss { max_age: Duration },
    Stale { revalidated: bool },
}

impl CacheReadStatus {
    pub fn into_headers(self) -> http::HeaderMap {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::HeaderName::from_static(X_GRAFBASE_CACHE),
            http::HeaderValue::from_static(match self {
                CacheReadStatus::Hit => "HIT",
                CacheReadStatus::Miss { .. } => "MISS",
                CacheReadStatus::Stale { revalidated } => {
                    if revalidated {
                        "UPDATING"
                    } else {
                        "STALE"
                    }
                }
                CacheReadStatus::Bypass => "BYPASS",
            }),
        );
        if let CacheReadStatus::Miss { max_age } = self {
            headers.typed_insert(headers::CacheControl::new().with_public().with_max_age(max_age));
        }
        headers
    }
}

/// Represents the response of the operation that was subject to caching
#[derive(Debug, PartialEq, Eq)]
pub enum CachedExecutionResponse<T> {
    /// The response is stale. It means we read a value from the cache but its considered stale
    /// `cache_revalidation` indicates if a revalidation request to the origin was issued
    Stale { response: T, cache_revalidation: bool },
    /// The response read from cache is still fresh
    Cached(T),
    /// We issued the request to the origin and got a response back
    /// `cache_read` indicates the caching behaviour:
    ///   - CacheReadStatus::Miss indicates that there was no value in the cache when we attempted to read and the response should be cached for `max-age`
    ///   - CacheReadStatus::Bypass indicates that no caching should take place (read or write)
    Origin { response: T, cache_read: CacheReadStatus },
}

impl<T> CachedExecutionResponse<T> {
    pub fn into_response_and_headers(self) -> (T, http::HeaderMap) {
        match self {
            CachedExecutionResponse::Cached(response) => (response, CacheReadStatus::Hit.into_headers()),
            CachedExecutionResponse::Stale {
                response,
                cache_revalidation: revalidated,
            } => (response, CacheReadStatus::Stale { revalidated }.into_headers()),
            CachedExecutionResponse::Origin { response, cache_read } => (response, cache_read.into_headers()),
        }
    }
}

/// Global cache config
#[derive(Clone, Default)]
pub struct GlobalCacheConfig {
    pub common_cache_tags: Vec<String>,
    pub enabled: bool,
    pub subdomain: String,
}

#[derive(Clone)]
pub struct Cache {
    config: Arc<GlobalCacheConfig>,
    inner: Arc<dyn CacheInner>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, derive_more::Display)]
pub struct Key(String);

impl Key {
    // Used by gateway-core to avoid refactoring half the tests...
    pub fn unchecked_new(key: String) -> Self {
        Self(key)
    }
}

impl Cache {
    pub fn new(inner: impl CacheInner + 'static, config: GlobalCacheConfig) -> Self {
        Self {
            config: Arc::new(config),
            inner: Arc::new(inner),
        }
    }

    pub fn build_key(&self, id: &str) -> Key {
        Key(format!("https://{}/{}", self.config.subdomain, id))
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &Key) -> Result<Entry<T>> {
        self.get(key)
            .await?
            .try_map(|bytes| serde_json::from_slice(&bytes).map_err(|err| Error::Serialization(err.to_string())))
    }

    // Tried Msgpack, but it doesn't behave really well with engine_v1::Response...
    pub async fn put_json<T: Serialize + DeserializeOwned>(
        &self,
        key: &Key,
        state: EntryState,
        value: &T,
        metadata: CacheMetadata,
    ) -> Result<()> {
        let bytes = serde_json::to_vec(value).map_err(|err| Error::Serialization(err.to_string()))?;
        self.put(key, state, bytes, metadata).await
    }
}

impl std::ops::Deref for Cache {
    type Target = dyn CacheInner;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

#[async_trait::async_trait]
pub trait CacheInner: Send + Sync {
    async fn get(&self, key: &Key) -> Result<Entry<Vec<u8>>>;
    async fn put(&self, key: &Key, state: EntryState, value: Vec<u8>, metadata: CacheMetadata) -> Result<()>;
    async fn delete(&self, key: &Key) -> Result<()>;
    async fn purge_by_tags(&self, tags: Vec<String>) -> Result<()>;
    async fn purge_by_hostname(&self, hostname: String) -> Result<()>;
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CacheMetadata {
    pub max_age: Duration,
    pub stale_while_revalidate: Duration,
    pub tags: Vec<String>,
    pub should_purge_related: bool,
    pub should_cache: bool,
}

impl CacheMetadata {
    pub fn with_priority_tags(mut self, tags: &[String]) -> Self {
        let mut tags = tags.to_vec();
        tags.extend(self.tags);
        self.tags = tags;
        self
    }
}

// Clone should be cheap.
pub trait Cacheable: DeserializeOwned + Serialize + Send + Sync {
    fn metadata(&self) -> CacheMetadata;
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils {
    use super::*;

    #[async_trait::async_trait]
    pub trait FakeCache: Send + Sync {
        async fn get(&self, _key: &Key) -> Result<Entry<Vec<u8>>> {
            unimplemented!()
        }

        async fn put(&self, _key: &Key, _status: EntryState, _value: Vec<u8>, _metadata: CacheMetadata) -> Result<()> {
            unimplemented!()
        }

        async fn delete(&self, _key: &Key) -> Result<()> {
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
    impl<T: FakeCache> CacheInner for T {
        async fn get(&self, key: &Key) -> Result<Entry<Vec<u8>>> {
            self.get(key).await
        }

        async fn put(&self, key: &Key, status: EntryState, value: Vec<u8>, metadata: CacheMetadata) -> Result<()> {
            self.put(key, status, value, metadata).await
        }

        async fn delete(&self, key: &Key) -> Result<()> {
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

#[cfg(test)]
mod test {
    use crate::cache::CacheReadStatus;
    use http::header::CACHE_CONTROL;
    use http::{HeaderMap, HeaderName, HeaderValue};
    use std::time::Duration;

    #[test]
    fn test_cache_hit_http_header() {
        let headers = CacheReadStatus::Hit.into_headers();

        let expected = HeaderMap::from_iter([(
            HeaderName::from_static("x-grafbase-cache"),
            HeaderValue::from_static("HIT"),
        )]);

        assert_eq!(headers, expected);
    }

    #[test]
    fn test_cache_bypass_http_header() {
        let headers = CacheReadStatus::Bypass.into_headers();

        let expected = HeaderMap::from_iter([(
            HeaderName::from_static("x-grafbase-cache"),
            HeaderValue::from_static("BYPASS"),
        )]);

        assert_eq!(headers, expected);
    }

    #[test]
    fn test_cache_stale_http_header() {
        let headers = CacheReadStatus::Stale { revalidated: false }.into_headers();

        let expected = HeaderMap::from_iter([(
            HeaderName::from_static("x-grafbase-cache"),
            HeaderValue::from_static("STALE"),
        )]);

        assert_eq!(headers, expected);

        let headers = CacheReadStatus::Stale { revalidated: true }.into_headers();

        let expected = HeaderMap::from_iter([(
            HeaderName::from_static("x-grafbase-cache"),
            HeaderValue::from_static("UPDATING"),
        )]);

        assert_eq!(headers, expected);
    }

    #[test]
    fn test_cache_miss_http_header() {
        let headers = CacheReadStatus::Miss {
            max_age: Duration::from_secs(1),
        }
        .into_headers();

        let expected = HeaderMap::from_iter([
            (
                HeaderName::from_static("x-grafbase-cache"),
                HeaderValue::from_static("MISS"),
            ),
            (CACHE_CONTROL, HeaderValue::from_static("public, max-age=1")),
        ]);

        assert_eq!(headers, expected);
    }
}
