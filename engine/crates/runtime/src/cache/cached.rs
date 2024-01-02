use std::fmt::Display;
use std::future::Future;
use std::sync::Arc;

use futures_util::{FutureExt, TryFutureExt};
use tracing_futures::Instrument;

use crate::cache::{
    Cache, CacheReadStatus, Cacheable, CachedExecutionResponse, Entry, EntryState, GlobalCacheConfig,
    RequestCacheConfig,
};
use crate::context::RequestContext;

pub async fn cached_execution<Value, Error, ValueFut>(
    cache: Arc<impl Cache<Value = Value> + 'static + ?Sized>,
    global_config: &GlobalCacheConfig,
    request_cache_config: &RequestCacheConfig,
    cache_key: String,
    ctx: &impl RequestContext,
    execution: ValueFut,
) -> Result<CachedExecutionResponse<Arc<Value>>, Error>
where
    Value: Cacheable + 'static,
    Error: Display + Send,
    ValueFut: Future<Output = Result<Arc<Value>, Error>> + Send + 'static,
{
    cached(cache, global_config, request_cache_config, ctx, cache_key, execution).await
}

async fn cached<Value, Error, ValueFut>(
    cache: Arc<impl Cache<Value = Value> + 'static + ?Sized>,
    config: &GlobalCacheConfig,
    request_cache_config: &RequestCacheConfig,
    ctx: &impl RequestContext,
    key: String,
    value_fut: ValueFut,
) -> Result<CachedExecutionResponse<Arc<Value>>, Error>
where
    Value: Cacheable + 'static,
    Error: Display + Send,
    ValueFut: Future<Output = Result<Arc<Value>, Error>> + Send + 'static,
{
    // skip if the incoming request doesn't want cached values, forces origin revalidation
    let cached_value = if request_cache_config.cache_control.no_cache {
        Entry::Miss
    } else {
        cache
            .get(&key)
            .instrument(tracing::info_span!("cache_get", ray_id = ctx.ray_id()))
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Error loading {} from cache: {}", key, e);
                Entry::Miss
            })
    };

    match cached_value {
        Entry::Stale {
            response,
            state,
            is_early_stale,
        } => {
            let response = Arc::new(response);
            let mut revalidated = false;

            // we only want to issue a revalidation if one is not already in progress
            if state != EntryState::UpdateInProgress {
                revalidated = true;

                update_stale(
                    cache,
                    ctx,
                    key.clone(),
                    response.clone(),
                    value_fut,
                    config.common_cache_tags.clone(),
                )
                .await;
            }

            tracing::info!(ray_id = ctx.ray_id(), "Cache STALE - {} - {}", revalidated, key);

            // early stales mean early refreshes on our part
            // they shouldn't be considered as stale from a client perspective
            if is_early_stale {
                tracing::info!(ray_id = ctx.ray_id(), "Cache HIT - {}", key);
                return Ok(CachedExecutionResponse::Cached(response));
            }

            Ok(CachedExecutionResponse::Stale {
                response,
                cache_revalidation: revalidated,
            })
        }
        Entry::Hit(gql_response) => {
            tracing::info!(ray_id = ctx.ray_id(), "Cache HIT - {}", key);

            Ok(CachedExecutionResponse::Cached(Arc::new(gql_response)))
        }
        Entry::Miss => {
            tracing::info!(ray_id = ctx.ray_id(), "Cache MISS - {}", key);

            let origin_result = value_fut.await?;
            let origin_tags = origin_result.cache_tags_with_priority_tags(config.common_cache_tags.clone());

            if origin_result.should_purge_related() {
                let ray_id = ctx.ray_id().to_string();
                let cache = Arc::clone(&cache);
                let purge_cache_tags = origin_tags.clone();

                tracing::info!(ray_id, "Purging global cache by tags: {:?}", purge_cache_tags);

                ctx.wait_until(
                    async_runtime::make_send_on_wasm(async move {
                        if let Err(err) = cache
                            .purge_by_tags(purge_cache_tags)
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

            if origin_result.should_cache() && !request_cache_config.cache_control.no_store {
                let ray_id = ctx.ray_id().to_string();
                let put_value = origin_result.clone();
                let cache = Arc::clone(&cache);

                ctx.wait_until(
                    async_runtime::make_send_on_wasm(async move {
                        if let Err(err) = cache
                            .put(&key, EntryState::Fresh, put_value, origin_tags)
                            .instrument(tracing::info_span!("cache_put"))
                            .await
                        {
                            tracing::error!(ray_id, "Error cache PUT: {}", err);
                        }
                    })
                    .boxed(),
                )
                .await;

                let max_age = origin_result.max_age();
                return Ok(CachedExecutionResponse::Origin {
                    response: origin_result,
                    cache_read: Some(CacheReadStatus::Miss { max_age }),
                });
            }

            Ok(CachedExecutionResponse::Origin {
                response: origin_result,
                cache_read: Some(CacheReadStatus::Bypass),
            })
        }
    }
}

async fn update_stale<Value, Error, ValueFut>(
    cache: Arc<impl Cache<Value = Value> + 'static + ?Sized>,
    ctx: &impl RequestContext,
    key: String,
    existing_value: Arc<Value>,
    value_fut: ValueFut,
    priority_tags: Vec<String>,
) where
    Value: Cacheable + 'static,
    Error: Display + Send,
    ValueFut: Future<Output = Result<Arc<Value>, Error>> + Send + 'static,
{
    let ray_id = ctx.ray_id().to_string();
    let existing_tags = existing_value.cache_tags_with_priority_tags(priority_tags.clone());

    // refresh the cache async and update the existing entry state
    let cache = Arc::clone(&cache);
    ctx.wait_until(
        async_runtime::make_send_on_wasm(async move {
            let put_futures = cache
                .put(
                    &key,
                    EntryState::UpdateInProgress,
                    existing_value.clone(),
                    existing_tags.clone(),
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
                    tracing::info!(ray_id, "Successfully fetched new value for cache from origin");
                    let fresh_cache_tags = fresh_value.cache_tags_with_priority_tags(priority_tags);

                    let _ = cache
                        .put(&key, EntryState::Fresh, fresh_value, fresh_cache_tags)
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
                        .put(&key, EntryState::Stale, existing_value, existing_tags)
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

    use crate::cache::cached::cached_execution;
    use crate::cache::{
        CacheReadStatus, Cacheable, CachedExecutionResponse, Entry, EntryState, GlobalCacheConfig, RequestCacheConfig,
        Result,
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
        fn max_age(&self) -> Duration {
            Duration::from_secs(self.max_age_seconds as u64)
        }

        fn stale_while_revalidate(&self) -> Duration {
            Duration::from_secs(self.stale_seconds as u64)
        }

        fn cache_tags(&self) -> Vec<String> {
            self.tags.clone()
        }

        fn should_purge_related(&self) -> bool {
            self.operation_type == OperationType::Mutation && !self.tags.is_empty()
        }

        fn should_cache(&self) -> bool {
            self.operation_type != OperationType::Mutation
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
            type Value = Dummy;

            async fn get(&self, _key: &str) -> Result<Entry<Self::Value>> {
                Ok(Entry::Miss)
            }

            async fn put(
                &self,
                _key: &str,
                _state: EntryState,
                _value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> Result<()> {
                Ok(())
            }
        }

        let res = dummy("Hi!");
        let res2 = res.clone();
        let response = cached_execution(
            Arc::new(TestCache),
            &GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
            &RequestCacheConfig {
                enabled: true,
                cache_control: Default::default(),
            },
            "the_key".to_string(),
            &FakeRequestContext::default(),
            async { Ok::<_, Error>(res2) },
        )
        .await
        .unwrap();

        assert_eq!(
            response,
            CachedExecutionResponse::Origin {
                response: res,
                cache_read: Some(CacheReadStatus::Miss {
                    max_age: Duration::from_secs(1),
                }),
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
            type Value = Dummy;

            async fn get(&self, _key: &str) -> Result<Entry<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Miss)
            }

            async fn put(
                &self,
                _key: &str,
                _state: EntryState,
                _value: Arc<Self::Value>,
                _tags: Vec<String>,
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
        let response = cached_execution(
            Arc::new(TestCache),
            &GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
            &RequestCacheConfig {
                enabled: true,
                cache_control: Default::default(),
            },
            "the_key".to_string(),
            &request_context,
            async { Ok::<_, Error>(dummy2) },
        )
        .await
        .unwrap();

        request_context.wait_for_futures().await;

        // assert
        assert_eq!(
            response,
            CachedExecutionResponse::Origin {
                response: dummy,
                cache_read: Some(CacheReadStatus::Miss {
                    max_age: Duration::from_secs(2),
                }),
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
            type Value = Dummy;

            async fn get(&self, _key: &str) -> Result<Entry<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Hit(Dummy {
                    value: "cached".to_string(),
                    ..Default::default()
                }))
            }
        }

        let ctx = FakeRequestContext::default();
        let response = cached_execution(
            Arc::new(TestCache),
            &GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
            &RequestCacheConfig {
                enabled: true,
                cache_control: Default::default(),
            },
            "the_key".to_string(),
            &ctx,
            async {
                Ok::<_, Error>(Arc::new(Dummy {
                    value: "new".to_string(),
                    ..Default::default()
                }))
            },
        )
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
            get_calls: AtomicUsize,
            put_calls: tokio::sync::RwLock<Vec<(EntryState, Arc<Dummy>)>>,
        }
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _key: &str) -> Result<Entry<Self::Value>> {
                self.get_calls.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Stale {
                    response: Dummy {
                        value: "stale".to_string(),
                        ..Default::default()
                    },
                    state: EntryState::Fresh,
                    is_early_stale: false,
                })
            }

            async fn put(
                &self,
                _key: &str,
                state: EntryState,
                value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> Result<()> {
                self.put_calls.write().await.push((state, value));
                Ok(())
            }
        }

        let ctx = FakeRequestContext::default();
        let cache = Arc::new(TestCache::default());

        // act
        let response = cached_execution(
            cache.clone(),
            &GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
            &RequestCacheConfig {
                enabled: true,
                cache_control: Default::default(),
            },
            "the_key".to_string(),
            &ctx,
            async { Ok::<_, Error>(dummy("new")) },
        )
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
        assert_eq!(1, cache.get_calls.load(Ordering::SeqCst));
        assert_eq!(
            cache.put_calls.read().await.iter().cloned().collect::<HashSet<_>>(),
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
            get_calls: AtomicUsize,
            entry_stale: AtomicBool,
            put_calls: RwLock<Vec<(EntryState, Arc<Dummy>)>>,
        }
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _key: &str) -> Result<Entry<Self::Value>> {
                self.get_calls.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Stale {
                    response: Dummy {
                        value: "stale".to_string(),
                        ..Default::default()
                    },
                    state: EntryState::Fresh,
                    is_early_stale: false,
                })
            }

            async fn put(
                &self,
                _key: &str,
                state: EntryState,
                value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> Result<()> {
                self.put_calls.write().await.push((state, value));
                self.entry_stale
                    .swap(matches!(state, EntryState::Stale), Ordering::SeqCst);
                Ok(())
            }
        }

        let ctx = FakeRequestContext::default();
        let cache = Arc::new(TestCache::default());

        // act
        let response = cached_execution(
            cache.clone(),
            &GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
            &RequestCacheConfig {
                enabled: true,
                cache_control: Default::default(),
            },
            "the_key".to_string(),
            &ctx,
            async { Err(Error::Origin("failed_source".to_string())) },
        )
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
        assert_eq!(1, cache.get_calls.load(Ordering::SeqCst));
        assert!(cache.entry_stale.load(Ordering::SeqCst));
        assert_eq!(
            cache.put_calls.read().await.iter().cloned().collect::<HashSet<_>>(),
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
            type Value = Dummy;

            async fn get(&self, _key: &str) -> Result<Entry<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Stale {
                    response: Dummy {
                        value: "stale".to_string(),
                        ..Default::default()
                    },
                    state: EntryState::UpdateInProgress,
                    is_early_stale: false,
                })
            }
        }

        let cache = Arc::new(TestCache);
        let ctx = FakeRequestContext::default();

        // act
        let response = cached_execution(
            cache.clone(),
            &GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
            &RequestCacheConfig {
                enabled: true,
                cache_control: Default::default(),
            },
            "the_key".to_string(),
            &ctx,
            async { Err(Error::Origin("failed_source".to_string())) },
        )
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
            type Value = Dummy;

            async fn get(&self, _key: &str) -> Result<Entry<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Miss)
            }
        }

        let cache = Arc::new(TestCache);
        let ctx = FakeRequestContext::default();
        let dummy = Arc::new(Dummy {
            value: "new".to_string(),
            operation_type: OperationType::Mutation,
            ..Default::default()
        });
        let dummy2 = Arc::clone(&dummy);

        // act
        let response = cached_execution(
            cache.clone(),
            &GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec![],
                ..Default::default()
            },
            &RequestCacheConfig {
                enabled: true,
                cache_control: Default::default(),
            },
            "the_key".to_string(),
            &ctx,
            async { Ok::<_, Error>(dummy2) },
        )
        .await
        .unwrap();

        ctx.wait_for_futures().await;

        // assert
        assert_eq!(
            response,
            CachedExecutionResponse::Origin {
                response: dummy,
                cache_read: Some(CacheReadStatus::Bypass),
            }
        );
        assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_successfully_purge_global() {
        #[derive(Default)]
        struct TestCache {
            get_calls: AtomicUsize,
            purge_calls: RwLock<Vec<Vec<String>>>,
        }
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _key: &str) -> Result<Entry<Self::Value>> {
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
        let cache = Arc::new(TestCache::default());

        // act
        let response = cached_execution(
            cache.clone(),
            &GlobalCacheConfig {
                enabled: true,
                common_cache_tags: vec!["project".to_string()],
                ..Default::default()
            },
            &RequestCacheConfig {
                enabled: true,
                cache_control: Default::default(),
            },
            "the_key".to_string(),
            &ctx,
            async { Ok::<_, Error>(dummy2) },
        )
        .await
        .unwrap();
        ctx.wait_for_futures().await;

        // assert
        assert_eq!(
            response,
            CachedExecutionResponse::Origin {
                response: dummy,
                cache_read: Some(CacheReadStatus::Bypass),
            }
        );
        assert_eq!(cache.get_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            cache.purge_calls.read().await.clone(),
            vec![vec!["project".to_string(), "tag".to_string()]]
        );
    }
}
