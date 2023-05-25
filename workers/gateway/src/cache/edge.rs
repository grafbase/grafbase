use std::marker::PhantomData;
use std::sync::Arc;

use worker::{Cache, Date, Headers, Response};

use crate::cache::error::CacheError;
use crate::cache::{CacheEntryState, CacheProvider, CacheProviderResponse, CacheResult, Cacheable};

const STALE_AT_HEADER: &str = "stale_at";
const CACHE_TAG_HEADER: &str = "Cache-Tag";
const MAX_CACHE_TAG_HEADER_SIZE: usize = 16_000;

pub struct EdgeCache<T> {
    _cache_value: PhantomData<T>,
}

#[async_trait::async_trait(?Send)]
impl<T: Cacheable + 'static> CacheProvider for EdgeCache<T> {
    type Value = T;

    async fn get(cache_name: &str, key: &str) -> CacheResult<CacheProviderResponse<Self::Value>> {
        let cache = Cache::open(cache_name.to_string()).await;

        let cf_response = cache
            .get(key, true)
            .await
            .map_err(|err| CacheError::CacheGet(err.to_string()))?;

        async {
            match cf_response {
                Some(mut response) => {
                    let bytes = response
                        .bytes()
                        .await
                        .map_err(|err| CacheError::CacheGet(err.to_string()))?;
                    let gql_response = worker_utils::json_request::deserialize(bytes)?;

                    if response.is_stale() {
                        Ok(CacheProviderResponse::Stale {
                            response: gql_response,
                            is_updating: matches!(response.cache_state(), CacheEntryState::Updating),
                        })
                    } else {
                        Ok(CacheProviderResponse::Hit(gql_response))
                    }
                }
                None => Ok(CacheProviderResponse::Miss),
            }
        }
        .await
    }

    async fn put(
        cache_name: &str,
        ray_id: &str,
        key: &str,
        status: CacheEntryState,
        value: Arc<Self::Value>,
        tags: Vec<String>,
    ) -> CacheResult<()> {
        let now_millis = Date::now().as_millis();
        let mut cache_headers = Headers::new();
        let mut cache_ttl_seconds = value.ttl_seconds();
        let mut stale_at_millis = now_millis + value.max_age_seconds() as u64 * 1000;

        if !matches!(status, CacheEntryState::Fresh) {
            cache_ttl_seconds = value.stale_seconds();
            stale_at_millis = now_millis;
        }

        // don't cache, instead delete the key from cache immediately
        // edge case: may happen when the new value to be cached for a given key is now 0
        // if so, delete the existing key from cache
        // e.g: a customer deployed a cache change that affects the same key. The new config tells us we shouldn't use cache
        if cache_ttl_seconds == 0 {
            return EdgeCache::<T>::delete(cache_name, ray_id, key).await;
        }

        cache_headers
            .set(
                http::header::CACHE_CONTROL.as_str(),
                &format!("public, s-maxage={cache_ttl_seconds}"),
            )
            .map_err(|err| CacheError::CachePut(err.to_string()))?;

        // when the cache entry is considered stale
        cache_headers
            .set(STALE_AT_HEADER, &format!("{stale_at_millis}"))
            .map_err(|err| CacheError::CachePut(err.to_string()))?;

        // cache entry status
        cache_headers
            .set(http::header::CACHE_STATUS.as_str(), status.into())
            .map_err(|err| CacheError::CachePut(err.to_string()))?;

        // cache entry tags - cloudflare has a max 16KB on `Cache-Tag` values. lets make sure we don't hit that
        let mut tags_str = String::with_capacity(MAX_CACHE_TAG_HEADER_SIZE);
        for tag in tags {
            if tags_str.bytes().len() + tag.bytes().len() > MAX_CACHE_TAG_HEADER_SIZE {
                break;
            }

            tags_str.push_str(&tag);
            tags_str.push(',');
        }

        cache_headers
            .set(CACHE_TAG_HEADER, &tags_str)
            .map_err(|err| CacheError::CachePut(err.to_string()))?;

        log::info!(ray_id, "cache {key} headers {cache_headers:?}");

        let bytes = worker_utils::json_request::serialize(value.as_ref())?;
        let cached_response = Response::from_bytes(bytes)
            .map_err(|e| CacheError::CachePut(e.to_string()))?
            .with_headers(cache_headers);

        let cache = Cache::open(cache_name.to_string()).await;
        cache.put(key, cached_response).await.map_err(|err| {
            log::error!(ray_id, "Failed to put cache {key}: {err}");
            CacheError::CachePut(err.to_string())
        })
    }

    async fn delete(cache_name: &str, ray_id: &str, key: &str) -> CacheResult<()> {
        let cache = Cache::open(cache_name.to_string()).await;
        cache.delete(key, false).await.map(|_| ()).map_err(|err| {
            log::error!(ray_id, "Failed to delete cache {key}: {err}");
            CacheError::CacheDelete(err.to_string())
        })
    }
}

trait CacheResponseExt {
    fn is_stale(&self) -> bool;
    fn cache_state(&self) -> CacheEntryState;
}

impl CacheResponseExt for Response {
    fn is_stale(&self) -> bool {
        self.headers()
            .get(STALE_AT_HEADER)
            .map(|stale_at| {
                let stale_at_millis: u64 = stale_at
                    .map(|stale_at| stale_at.parse().unwrap_or_default())
                    .unwrap_or_default();
                let now = Date::now().as_millis();

                now > stale_at_millis
            })
            .unwrap_or(true) // if anything goes wrong say its stale
    }

    fn cache_state(&self) -> CacheEntryState {
        self.headers()
            .get(http::header::CACHE_STATUS.as_str())
            .map(|cache_status| {
                cache_status
                    .and_then(|cache_status| cache_status.parse().ok())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }
}
