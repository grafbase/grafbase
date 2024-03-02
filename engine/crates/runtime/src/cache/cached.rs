use std::borrow::Cow;
use std::future::Future;
use std::time::Duration;

use bytes::Bytes;

use crate::{
    bytes::OwnedOrSharedBytes,
    cache::{Cache, Entry, UPDATE_IN_PROGRESS_NAMEPACE},
};

use super::{CacheError, CacheStatus, OperationCacheControl, VALUE_NAMESPACE};

const UPDATE_IN_PROGRESS_MAX_AGE: Duration = Duration::from_secs(3);

pub struct CachedResponse {
    pub status: CacheStatus,
    pub cache_control: headers::CacheControl,
    pub body: OwnedOrSharedBytes,
}

pub struct TaggedResponseContent {
    pub body: Vec<u8>,
    pub cache_tags: Vec<String>,
}

impl Cache {
    pub async fn cached_execution<Fut, Error>(
        &self,
        key: &str,
        request_cache_control: Option<headers::CacheControl>,
        operation_cache_control: OperationCacheControl,
        execution: Fut,
    ) -> Result<CachedResponse, Error>
    where
        Fut: Future<Output = Result<TaggedResponseContent, Error>> + Send + 'static,
        Error: Send,
    {
        let (no_cache, no_store) = match request_cache_control {
            Some(cc) => (cc.no_cache(), cc.no_store()),
            None => (false, false),
        };
        let cached_value: Entry<Vec<u8>> = if no_cache {
            Entry::Miss
        } else {
            self.raw.get(VALUE_NAMESPACE, key).await.unwrap_or_else(|e| {
                tracing::warn!("Error loading {} from cache: {}", key, e);
                Entry::Miss
            })
        };

        match cached_value {
            Entry::Hit {
                value,
                stale_at,
                invalid_at,
            } => {
                let now = std::time::Instant::now();
                Ok(CachedResponse {
                    status: CacheStatus::Hit,
                    cache_control: headers::CacheControl::new()
                        // Only public responses are cached by us.
                        .with_public()
                        .with_max_age(stale_at.checked_duration_since(now).unwrap_or_default())
                        .with_max_stale(invalid_at.checked_duration_since(now).unwrap_or_default()),
                    body: value.into(),
                })
            }
            Entry::Stale { value, invalid_at } => {
                if !no_store {
                    let cache = self.clone();
                    let key = key.to_string();
                    self.async_runtime.spawn_faillible::<CacheError>(async move {
                        // update is already in progress
                        if let Ok(Entry::Hit { .. }) = cache.raw.get(UPDATE_IN_PROGRESS_NAMEPACE, &key).await {
                            return Ok(());
                        }
                        cache
                            .raw
                            .put(
                                UPDATE_IN_PROGRESS_NAMEPACE,
                                &key,
                                Cow::Owned(Vec::new()),
                                Vec::new(),
                                UPDATE_IN_PROGRESS_MAX_AGE,
                                Duration::from_secs(0),
                            )
                            .await?;
                        let Ok(TaggedResponseContent { body, cache_tags }) = execution.await else {
                            return Ok(());
                        };

                        if !operation_cache_control.is_private() {
                            cache
                                .put(&key, body, operation_cache_control.max_age)
                                .with_tags(cache_tags)
                                .with_max_stale(operation_cache_control.max_stale)
                                .await?;
                        }

                        Ok(())
                    })
                }

                let now = std::time::Instant::now();
                Ok(CachedResponse {
                    status: CacheStatus::Stale,
                    cache_control: headers::CacheControl::new()
                        // Only public responses are cached by us.
                        .with_public()
                        .with_max_age(Duration::ZERO)
                        .with_max_stale(invalid_at.checked_duration_since(now).unwrap_or_default()),

                    body: value.into(),
                })
            }
            Entry::Miss => {
                let TaggedResponseContent { body, cache_tags: tags } = execution.await?;

                let body = Bytes::from(body);
                let response_cache_control = operation_cache_control.to_response_header();
                if !(no_store || operation_cache_control.is_private() || operation_cache_control.max_age.is_zero()) {
                    let value = body.clone();
                    let cache = self.clone();
                    let key = key.to_string();
                    self.async_runtime.spawn_faillible(async move {
                        cache
                            .put(&key, value.as_ref(), operation_cache_control.max_age)
                            .with_tags(tags)
                            .with_max_stale(operation_cache_control.max_stale)
                            .await
                    })
                }
                Ok(CachedResponse {
                    status: CacheStatus::Miss,
                    cache_control: response_cache_control,
                    body: body.into(),
                })
            }
        }
    }
}
