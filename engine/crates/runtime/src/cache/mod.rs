mod cached;

use bytes::Bytes;
use headers::HeaderMapExt;
use std::{sync::Arc, time::Duration};

use serde::{de::DeserializeOwned, Serialize};

const X_GRAFBASE_CACHE: &str = "x-grafbase-cache";

pub type Result<T> = std::result::Result<T, Error>;

pub use cached::cached_execution;

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
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StaleEntry<T> {
    value: T,
    state: EntryState,
    is_early_stale: bool,
    metadata: CacheMetadata,
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
    Origin {
        response: T,
        cache_read: Option<CacheReadStatus>,
    },
}

#[derive(Clone, Default)]
pub struct CacheControl {
    /// The no-cache request directive asks caches to validate the response with the origin server before reuse.
    /// no-cache allows clients to request the most up-to-date response even if the cache has a fresh response.
    pub no_cache: bool,
    /// The no-store request directive allows a client to request that caches refrain from storing
    /// the request and corresponding response — even if the origin server's response could be stored.
    pub no_store: bool,
}

/// Global cache config
#[derive(Clone, Default)]
pub struct GlobalCacheConfig {
    pub common_cache_tags: Vec<String>,
    pub enabled: bool,
    pub subdomain: String,
}

/// Request cache config
#[derive(Clone, Default)]
pub struct RequestCacheConfig {
    pub enabled: bool,
    pub cache_control: CacheControl,
}

#[derive(Clone)]
pub struct Cache(Arc<dyn CacheInner>);

impl Cache {
    pub fn new(inner: impl CacheInner + 'static) -> Self {
        Self(Arc::new(inner))
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Entry<T>> {
        self.get(key)
            .await?
            .try_map(|bytes| serde_json::from_slice(&bytes).map_err(|err| Error::Serialization(err.to_string())))
    }

    pub async fn put_json<T: serde::Serialize>(
        &self,
        key: &str,
        state: EntryState,
        value: &T,
        metadata: CacheMetadata,
    ) -> Result<()> {
        let bytes = serde_json::to_vec(value).map_err(|err| Error::Serialization(err.to_string()))?;
        self.put(key, state, bytes.into(), metadata).await
    }
}

impl std::ops::Deref for Cache {
    type Target = dyn CacheInner;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[async_trait::async_trait]
pub trait CacheInner: Send + Sync {
    async fn get(&self, key: &str) -> Result<Entry<Bytes>>;
    async fn put(&self, key: &str, state: EntryState, value: Bytes, metadata: CacheMetadata) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn purge_by_tags(&self, tags: Vec<String>) -> Result<()>;
    async fn purge_by_hostname(&self, hostname: String) -> Result<()>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheMetadata {
    max_age: Duration,
    stale_while_revalidate: Duration,
    tags: Vec<String>,
    should_purge_related: bool,
    should_cache: bool,
}

impl CacheMetadata {
    fn with_priority_tags(mut self, tags: &Vec<String>) -> Self {
        let mut tags = tags.clone();
        tags.extend(self.tags);
        self.tags = tags;
        self
    }
}

pub trait Cacheable: DeserializeOwned + Serialize + Send + Sync {
    // Also retrieved during cache.get(), so needs to be included in the value.
    fn max_age(&self) -> Duration;
    fn stale_while_revalidate(&self) -> Duration;
    fn cache_tags(&self) -> Vec<String>;
    fn should_purge_related(&self) -> bool;
    fn should_cache(&self) -> bool;
}

#[cfg(any(test, feature = "test-utils"))]
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
