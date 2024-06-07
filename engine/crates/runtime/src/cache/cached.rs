use futures_util::{FutureExt, TryFutureExt};
use headers::HeaderMapExt;
use std::fmt::Display;
use std::future::Future;
use std::sync::Arc;
use tracing_futures::Instrument;

use crate::cache::{Cache, CacheReadStatus, Cacheable, CachedExecutionResponse, Entry, EntryState};
use crate::context::RequestContext;

use super::{CacheMetadata, Key};

impl Cache {
    pub async fn cached_execution<Value, Error, ValueFut>(
        &self,
        ctx: &impl RequestContext,
        key: Key,
        execution: ValueFut,
    ) -> Result<CachedExecutionResponse<Arc<Value>>, Error>
    where
        Value: Cacheable + 'static,
        Error: Display + Send,
        ValueFut: Future<Output = Result<Arc<Value>, Error>> + Send + 'static,
    {
        let cache_control = ctx
            .headers()
            .typed_get::<headers::CacheControl>()
            .unwrap_or_else(headers::CacheControl::new);
        let cache_span =
            grafbase_tracing::span::cache::CacheSpan::new(CacheReadStatus::Bypass.to_header_value()).into_span();

        if self.config.enabled {
            cached(self, cache_control, ctx, key, execution)
                .inspect_ok(|cached_response| {
                    use grafbase_tracing::span::CacheRecorderSpanExt;

                    cache_span.record_status(cached_response.read_status().to_header_value());
                })
                .inspect_err(|_| {
                    use grafbase_tracing::span::CacheRecorderSpanExt;

                    cache_span.record_error();
                })
                .instrument(cache_span.clone())
                .await
        } else {
            Ok(CachedExecutionResponse::Origin {
                response: execution.await?,
                cache_read: CacheReadStatus::Bypass,
            })
        }
    }
}

async fn cached<Value, Error, ValueFut>(
    cache: &Cache,
    cache_control: headers::CacheControl,
    ctx: &impl RequestContext,
    key: Key,
    value_fut: ValueFut,
) -> Result<CachedExecutionResponse<Arc<Value>>, Error>
where
    Value: Cacheable + 'static,
    Error: Display + Send,
    ValueFut: Future<Output = Result<Arc<Value>, Error>> + Send + 'static,
{
    // skip if the incoming request doesn't want cached values, forces origin revalidation
    let cached_value: Entry<Value> = if cache_control.no_cache() {
        Entry::Miss
    } else {
        cache
            .get_json(&key)
            .instrument(tracing::info_span!("cache_get", ray_id = ctx.ray_id()))
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Error loading {} from cache: {}", key, e);
                Entry::Miss
            })
    };

    match cached_value {
        Entry::Stale(entry) => {
            let mut revalidated = false;
            let value = Arc::new(entry.value);

            // we only want to issue a revalidation if one is not already in progress
            if entry.state != EntryState::UpdateInProgress {
                revalidated = true;

                update_stale(cache, ctx, key.clone(), Arc::clone(&value), entry.metadata, value_fut).await;
            }

            tracing::debug!(ray_id = ctx.ray_id(), "Cache STALE - {} - {}", revalidated, key);

            // early stales mean early refreshes on our part
            // they shouldn't be considered as stale from a client perspective
            if entry.is_early_stale {
                tracing::debug!(ray_id = ctx.ray_id(), "Cache HIT - {}", key);
                return Ok(CachedExecutionResponse::Cached(value));
            }

            Ok(CachedExecutionResponse::Stale {
                response: value,
                cache_revalidation: revalidated,
            })
        }
        Entry::Hit(value, _) => {
            tracing::debug!(ray_id = ctx.ray_id(), "Cache HIT - {}", key);

            Ok(CachedExecutionResponse::Cached(Arc::new(value)))
        }
        Entry::Miss => {
            tracing::debug!(ray_id = ctx.ray_id(), "Cache MISS - {}", key);

            let origin_result = value_fut.await?;
            let metadata = origin_result
                .metadata()
                .with_priority_tags(&cache.config.common_cache_tags);

            if metadata.should_purge_related {
                let ray_id = ctx.ray_id().to_string();
                let cache = cache.clone();
                let purge_tags = metadata.tags.clone();

                tracing::debug!(ray_id, "Purging global cache by tags: {:?}", purge_tags);

                ctx.wait_until(
                    async_runtime::make_send_on_wasm(async move {
                        if let Err(err) = cache
                            .purge_by_tags(purge_tags)
                            .instrument(tracing::info_span!("cache_tags_purge", ray_id))
                            .await
                        {
                            tracing::error!(ray_id, "Error global cache purge by tags: {}", err);
                        }
                    })
                    .boxed(),
                )
                .await;
            }

            if metadata.should_cache && !cache_control.no_store() {
                let ray_id = ctx.ray_id().to_string();
                let put_value = origin_result.clone();
                let max_age = metadata.max_age;
                let cache = cache.clone();

                ctx.wait_until(
                    async_runtime::make_send_on_wasm(async move {
                        if let Err(err) = cache
                            .put_json(&key, EntryState::Fresh, put_value.as_ref(), metadata)
                            .instrument(tracing::info_span!("cache_put"))
                            .await
                        {
                            tracing::error!(ray_id, "Error cache PUT: {}", err);
                        }
                    })
                    .boxed(),
                )
                .await;

                return Ok(CachedExecutionResponse::Origin {
                    response: origin_result,
                    cache_read: CacheReadStatus::Miss { max_age },
                });
            }

            Ok(CachedExecutionResponse::Origin {
                response: origin_result,
                cache_read: CacheReadStatus::Bypass,
            })
        }
    }
}

async fn update_stale<Value, Error, ValueFut>(
    cache: &Cache,
    ctx: &impl RequestContext,
    key: Key,
    existing_value: Arc<Value>,
    existing_metadata: CacheMetadata,
    value_fut: ValueFut,
) where
    Value: Cacheable + 'static,
    Error: Display + Send,
    ValueFut: Future<Output = Result<Arc<Value>, Error>> + Send + 'static,
{
    let ray_id = ctx.ray_id().to_string();
    let existing_metadata = existing_metadata.with_priority_tags(&cache.config.common_cache_tags);

    // refresh the cache async and update the existing entry state
    let cache = cache.clone();
    ctx.wait_until(
        async_runtime::make_send_on_wasm(async move {
            let put_futures = cache
                .put_json(
                    &key,
                    EntryState::UpdateInProgress,
                    existing_value.as_ref(),
                    existing_metadata.clone(),
                )
                .instrument(tracing::info_span!("cache_put_updating"))
                .inspect_err(|err| {
                    tracing::error!(
                        ray_id,
                        "Error transitioning cache entry to {} - {}",
                        EntryState::UpdateInProgress,
                        err
                    );
                });

            let (_, source_result) = futures_util::join!(put_futures, value_fut);

            match source_result {
                Ok(fresh_value) => {
                    tracing::debug!(ray_id, "Successfully fetched new value for cache from origin");
                    let fresh_metadata = fresh_value
                        .metadata()
                        .with_priority_tags(&cache.config.common_cache_tags);

                    let _ = cache
                        .put_json(&key, EntryState::Fresh, fresh_value.as_ref(), fresh_metadata)
                        .instrument(tracing::info_span!("cache_put_refresh"))
                        .inspect_err(|err| {
                            // if this errors we're probably stuck in `UPDATING` state or
                            // we're serving a cache entries as `FRESH` where in reality they are stale (in case the `UPDATING` transition failed)
                            tracing::error!(ray_id, "Error updating stale cache entry with fresh value - {}", err);
                        })
                        .await;
                }
                Err(err) => {
                    tracing::error!(ray_id, "Error fetching fresh value for a stale cache entry: {}", err);
                    let _ = cache
                        .put_json(&key, EntryState::Stale, existing_value.as_ref(), existing_metadata)
                        .instrument(tracing::info_span!("cache_put_stale"))
                        .inspect_err(|err| {
                            // if this errors we're probably stuck in `UPDATING` state or
                            // we're serving cache entries as `FRESH` when in reality they are stale (in case the `UPDATING` transition failed)
                            tracing::error!(
                                ray_id,
                                "Error transitioning cache entry to {} - {}",
                                EntryState::Stale,
                                err,
                            );
                        })
                        .await;
                }
            };
        })
        .boxed(),
    )
    .await;
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use futures_util::future::BoxFuture;
    use tokio::sync::{Mutex, RwLock};

    use common_types::OperationType;

    use crate::cache::{
        Cache, CacheMetadata, CacheReadStatus, Cacheable, CachedExecutionResponse, Entry, EntryState,
        GlobalCacheConfig, Key, Result, StaleEntry,
    };
    use crate::context::RequestContext;
    use crate::{cache::test_utils::FakeCache, cache::Error};

    #[derive(serde::Deserialize, serde::Serialize, Clone, Eq, PartialEq, Debug, Hash)]
    struct Dummy {
        value: String,
        max_age_seconds: usize,
        stale_seconds: usize,
        operation_type: OperationType,
        tags: Vec<String>,
    }

    impl Default for Dummy {
        fn default() -> Self {
            Self {
                value: String::new(),
                max_age_seconds: 1,
                stale_seconds: 2,
                operation_type: OperationType::Query {
                    is_introspection: false,
                },
                tags: vec![],
            }
        }
    }

    fn dummy(value: &'static str) -> Arc<Dummy> {
        Arc::new(Dummy {
            value: value.to_string(),
            ..Default::default()
        })
    }

    impl Dummy {
        fn new(n: usize, operation_type: OperationType, tags: Vec<String>) -> Self {
            Self {
                max_age_seconds: n,
                stale_seconds: n,
                operation_type,
                tags,
                ..Default::default()
            }
        }
    }

    impl Cacheable for Dummy {
        fn metadata(&self) -> CacheMetadata {
            CacheMetadata {
                max_age: Duration::from_secs(self.max_age_seconds as u64),
                stale_while_revalidate: Duration::from_secs(self.stale_seconds as u64),
                tags: self.tags.clone(),
                should_purge_related: self.operation_type == OperationType::Mutation && !self.tags.is_empty(),
                should_cache: self.operation_type != OperationType::Mutation,
            }
        }
    }

    #[derive(Default)]
    struct FakeRequestContext {
        headers: http::HeaderMap,
        futures: Mutex<Vec<BoxFuture<'static, ()>>>,
    }

    impl FakeRequestContext {
        async fn wait_for_futures(&self) {
            let futures = self
                .futures
                .lock()
                .await
                .drain(..)
                .collect::<Vec<BoxFuture<'static, ()>>>();
            futures_util::future::join_all(futures).await;
        }
    }

    #[async_trait::async_trait]
    impl RequestContext for FakeRequestContext {
        fn ray_id(&self) -> &str {
            "ray-id"
        }

        async fn wait_until(&self, fut: BoxFuture<'static, ()>) {
            self.futures.lock().await.push(fut);
        }

        fn headers(&self) -> &http::HeaderMap {
            &self.headers
        }
    }

    #[tokio::test]
    async fn should_execute_with_cache_miss() {
        struct TestCache;
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            async fn get(&self, _key: &Key) -> Result<Entry<Vec<u8>>> {
                Ok(Entry::Miss)
            }

            async fn put(
                &self,
                _key: &Key,
                _state: EntryState,
                _value: Vec<u8>,
                _metadata: CacheMetadata,
            ) -> Result<()> {
                Ok(())
            }
        }

        let res = dummy("Hi!");
        let res2 = res.clone();
        let cache = Cache::new(
            TestCache,
            GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
        );
        let response = cache
            .cached_execution(&FakeRequestContext::default(), cache.build_key("the_key"), async {
                Ok::<_, Error>(res2)
            })
            .await
            .unwrap();

        assert_eq!(
            response,
            CachedExecutionResponse::Origin {
                response: res,
                cache_read: CacheReadStatus::Miss {
                    max_age: Duration::from_secs(1),
                },
            }
        );
    }

    #[tokio::test]
    async fn should_successfully_get_miss() {
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);
        static PUT_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            async fn get(&self, _key: &Key) -> Result<Entry<Vec<u8>>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Miss)
            }

            async fn put(
                &self,
                _key: &Key,
                _state: EntryState,
                _value: Vec<u8>,
                _metadata: CacheMetadata,
            ) -> Result<()> {
                PUT_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let dummy = Arc::new(Dummy::new(
            2,
            OperationType::Query {
                is_introspection: false,
            },
            vec![],
        ));
        let dummy2 = Arc::clone(&dummy);

        // act
        let request_context = FakeRequestContext::default();
        let cache = Cache::new(
            TestCache,
            GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
        );
        let response = cache
            .cached_execution(&request_context, cache.build_key("the_key"), async {
                Ok::<_, Error>(dummy2)
            })
            .await
            .unwrap();

        request_context.wait_for_futures().await;

        // assert
        assert_eq!(
            response,
            CachedExecutionResponse::Origin {
                response: dummy,
                cache_read: CacheReadStatus::Miss {
                    max_age: Duration::from_secs(2),
                },
            }
        );
        assert_eq!(1, PUT_CALLS.load(Ordering::SeqCst));
        assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_successfully_get_hit() {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            async fn get(&self, _key: &Key) -> Result<Entry<Vec<u8>>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Hit(
                    serde_json::to_vec(&Dummy {
                        value: "cached".to_string(),
                        ..Default::default()
                    })
                    .unwrap(),
                    Duration::from_millis(500),
                ))
            }
        }

        let ctx = FakeRequestContext::default();
        let cache = Cache::new(
            TestCache,
            GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
        );
        let response = cache
            .cached_execution(&ctx, cache.build_key("the_key"), async {
                Ok::<_, Error>(Arc::new(Dummy {
                    value: "new".to_string(),
                    ..Default::default()
                }))
            })
            .await
            .unwrap();
        ctx.wait_for_futures().await;

        assert_eq!(response, CachedExecutionResponse::Cached(dummy("cached")));
        assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_successfully_update_stale() {
        #[derive(Default)]
        struct TestCache {
            get_calls: Arc<AtomicUsize>,
            #[allow(clippy::type_complexity)]
            put_calls: Arc<RwLock<Vec<(EntryState, Arc<Dummy>)>>>,
        }

        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            async fn get(&self, _key: &Key) -> Result<Entry<Vec<u8>>> {
                self.get_calls.fetch_add(1, Ordering::SeqCst);
                let dummy = Dummy {
                    value: "stale".to_string(),
                    ..Default::default()
                };
                Ok(Entry::Stale(StaleEntry {
                    value: serde_json::to_vec(&dummy).unwrap(),
                    state: EntryState::Fresh,
                    is_early_stale: false,
                    metadata: dummy.metadata(),
                }))
            }

            async fn put(&self, _key: &Key, state: EntryState, value: Vec<u8>, _metadata: CacheMetadata) -> Result<()> {
                self.put_calls
                    .write()
                    .await
                    .push((state, Arc::new(serde_json::from_slice(&value).unwrap())));
                Ok(())
            }
        }

        let ctx = FakeRequestContext::default();
        let cache = TestCache::default();
        let get_calls = cache.get_calls.clone();
        let put_calls = cache.put_calls.clone();
        let cache = Cache::new(
            cache,
            GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
        );

        // act
        let response = cache
            .cached_execution(&ctx, cache.build_key("the_key"), async { Ok::<_, Error>(dummy("new")) })
            .await
            .unwrap();

        ctx.wait_for_futures().await;

        // assert
        assert_eq!(
            response,
            CachedExecutionResponse::Stale {
                response: dummy("stale"),
                cache_revalidation: true,
            }
        );
        assert_eq!(1, get_calls.load(Ordering::SeqCst));
        assert_eq!(
            put_calls.read().await.iter().cloned().collect::<HashSet<_>>(),
            HashSet::from([
                (EntryState::UpdateInProgress, dummy("stale")),
                (EntryState::Fresh, dummy("new"))
            ])
        );
    }

    #[tokio::test]
    async fn should_fail_update_stale() {
        #[derive(Default)]
        struct TestCache {
            get_calls: Arc<AtomicUsize>,
            entry_stale: Arc<AtomicBool>,
            #[allow(clippy::type_complexity)]
            put_calls: Arc<RwLock<Vec<(EntryState, Arc<Dummy>)>>>,
        }
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            async fn get(&self, _key: &Key) -> Result<Entry<Vec<u8>>> {
                self.get_calls.fetch_add(1, Ordering::SeqCst);
                let dummy = Dummy {
                    value: "stale".to_string(),
                    ..Default::default()
                };
                Ok(Entry::Stale(StaleEntry {
                    value: serde_json::to_vec(&dummy).unwrap(),
                    state: EntryState::Fresh,
                    is_early_stale: false,
                    metadata: dummy.metadata(),
                }))
            }

            async fn put(&self, _key: &Key, state: EntryState, value: Vec<u8>, _metadata: CacheMetadata) -> Result<()> {
                self.put_calls
                    .write()
                    .await
                    .push((state, Arc::new(serde_json::from_slice(&value).unwrap())));
                self.entry_stale
                    .swap(matches!(state, EntryState::Stale), Ordering::SeqCst);
                Ok(())
            }
        }

        let ctx = FakeRequestContext::default();
        let cache = TestCache::default();
        let get_calls = cache.get_calls.clone();
        let put_calls = cache.put_calls.clone();
        let entry_stale = cache.entry_stale.clone();
        let cache = Cache::new(
            cache,
            GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
        );

        // act
        let response = cache
            .cached_execution(&ctx, cache.build_key("the_key"), async {
                Err(Error::Origin("failed_source".to_string()))
            })
            .await
            .unwrap();

        ctx.wait_for_futures().await;

        // assert
        assert_eq!(
            response,
            CachedExecutionResponse::Stale {
                response: dummy("stale"),
                cache_revalidation: true,
            }
        );
        assert_eq!(1, get_calls.load(Ordering::SeqCst));
        assert!(entry_stale.load(Ordering::SeqCst));
        assert_eq!(
            put_calls.read().await.iter().cloned().collect::<HashSet<_>>(),
            HashSet::from([
                (EntryState::UpdateInProgress, dummy("stale")),
                (EntryState::Stale, dummy("stale"))
            ])
        );
    }

    #[tokio::test]
    async fn should_not_update_stale_entry_when_updating() {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            async fn get(&self, _key: &Key) -> Result<Entry<Vec<u8>>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                let dummy = Dummy {
                    value: "stale".to_string(),
                    ..Default::default()
                };
                Ok(Entry::Stale(StaleEntry {
                    value: serde_json::to_vec(&dummy).unwrap(),
                    state: EntryState::UpdateInProgress,
                    is_early_stale: false,
                    metadata: dummy.metadata(),
                }))
            }
        }

        let cache = Cache::new(
            TestCache,
            GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
        );
        let ctx = FakeRequestContext::default();

        // act
        let response = cache
            .cached_execution(&ctx, cache.build_key("the_key"), async {
                Err(Error::Origin("failed_source".to_string()))
            })
            .await
            .unwrap();
        ctx.wait_for_futures().await;

        // assert
        assert_eq!(
            response,
            CachedExecutionResponse::Stale {
                response: dummy("stale"),
                cache_revalidation: false,
            }
        );
        assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_successfully_get_bypass() {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            async fn get(&self, _key: &Key) -> Result<Entry<Vec<u8>>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Miss)
            }
        }

        let cache = Cache::new(
            TestCache,
            GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
        );
        let ctx = FakeRequestContext::default();
        let dummy = Arc::new(Dummy {
            value: "new".to_string(),
            operation_type: OperationType::Mutation,
            ..Default::default()
        });
        let dummy2 = Arc::clone(&dummy);

        // act
        let response = cache
            .cached_execution(&ctx, cache.build_key("the_key"), async { Ok::<_, Error>(dummy2) })
            .await
            .unwrap();

        ctx.wait_for_futures().await;

        // assert
        assert_eq!(
            response,
            CachedExecutionResponse::Origin {
                response: dummy,
                cache_read: CacheReadStatus::Bypass,
            }
        );
        assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_successfully_purge_global() {
        #[derive(Default)]
        struct TestCache {
            get_calls: Arc<AtomicUsize>,
            purge_calls: Arc<RwLock<Vec<Vec<String>>>>,
        }
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            async fn get(&self, _key: &Key) -> Result<Entry<Vec<u8>>> {
                self.get_calls.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Miss)
            }

            async fn purge_by_tags(&self, tags: Vec<String>) -> Result<()> {
                self.purge_calls.write().await.push(tags);
                Ok(())
            }
        }

        let ctx = FakeRequestContext::default();
        let dummy = Arc::new(Dummy {
            value: "new".to_string(),
            operation_type: OperationType::Mutation,
            tags: vec!["tag".into()],
            ..Default::default()
        });
        let dummy2 = Arc::clone(&dummy);
        let cache = TestCache::default();
        let get_calls = cache.get_calls.clone();
        let purge_calls = cache.purge_calls.clone();
        let cache = Cache::new(
            cache,
            GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec!["project".to_string()],
                ..Default::default()
            },
        );

        // act
        let response = cache
            .cached_execution(&ctx, cache.build_key("the_key"), async { Ok::<_, Error>(dummy2) })
            .await
            .unwrap();
        ctx.wait_for_futures().await;

        // assert
        assert_eq!(
            response,
            CachedExecutionResponse::Origin {
                response: dummy,
                cache_read: CacheReadStatus::Bypass,
            }
        );
        assert_eq!(get_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            purge_calls.read().await.clone(),
            vec![vec!["project".to_string(), "tag".to_string()]]
        );
    }
}
