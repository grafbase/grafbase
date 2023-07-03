use futures_util::{FutureExt, TryFutureExt};
use send_wrapper::SendWrapper;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use tracing_futures::Instrument;

use crate::cache::error::CacheError;
use crate::cache::{
    CacheEntryState, CacheProvider, CacheProviderResponse, CacheResult, Cacheable, GlobalCacheProvider,
};
use crate::platform::context::RequestContext;

pub enum CacheResponse<Type> {
    Hit(Type),
    Miss(Type),
    Bypass(Type),
    Stale { response: Type, is_updating: bool },
}

pub struct Cache<CV, Provider> {
    cache_name: String,
    global_cache: Arc<Box<dyn GlobalCacheProvider>>,
    _cache_value: PhantomData<CV>,
    _cache_provider: PhantomData<Provider>,
}

impl<CV: Cacheable + 'static, P: CacheProvider<Value = CV>> Cache<CV, P> {
    pub fn new(cache_name: String, global_cache: Box<dyn GlobalCacheProvider>) -> Self {
        Cache {
            cache_name,
            global_cache: Arc::new(global_cache),
            _cache_value: PhantomData,
            _cache_provider: PhantomData,
        }
    }

    pub async fn cached(
        &self,
        request_context: &RequestContext,
        cache_key: &str,
        source_future: impl Future<Output = worker::Result<CV>> + 'static,
    ) -> CacheResult<CacheResponse<Arc<CV>>>
    where
        CV: Default,
    {
        // skip if the incoming request doesn't want cached values, forces origin revalidation
        let cached_value: CacheProviderResponse<CV> =
            if request_context.cloudflare_request_context.cache_control.no_cache {
                CacheProviderResponse::Miss
            } else {
                P::get(&self.cache_name, cache_key)
                    .instrument(tracing::info_span!("cache_get"))
                    .await
                    .unwrap_or_else(|e| {
                        log::warn!(
                            request_context.cloudflare_request_context.ray_id,
                            "Error loading {cache_key} from cache: {e}",
                        );
                        CacheProviderResponse::Miss
                    })
            };

        let mut priority_cache_tags = vec![request_context.config.customer_deployment_config.project_id.clone()];
        if let Some(branch) = &request_context.config.customer_deployment_config.github_ref_name {
            priority_cache_tags.insert(0, branch.clone());
        }

        match cached_value {
            CacheProviderResponse::Stale {
                response,
                mut is_updating,
            } => {
                log::info!(
                    request_context.cloudflare_request_context.ray_id,
                    "Cache STALE - {cache_key}"
                );

                let response_arc = Arc::new(response);
                if !is_updating {
                    is_updating = self
                        .update_stale(
                            request_context,
                            cache_key,
                            response_arc.clone(),
                            source_future,
                            priority_cache_tags,
                        )
                        .await?;
                }

                Ok(CacheResponse::Stale {
                    response: response_arc,
                    is_updating,
                })
            }
            CacheProviderResponse::Hit(gql_response) => {
                log::info!(
                    request_context.cloudflare_request_context.ray_id,
                    "Cache HIT - {cache_key}"
                );

                Ok(CacheResponse::Hit(Arc::new(gql_response)))
            }
            CacheProviderResponse::Miss => {
                log::info!(
                    request_context.cloudflare_request_context.ray_id,
                    "Cache MISS - {cache_key}"
                );

                let origin_result = Arc::new(source_future.await.map_err(CacheError::Origin)?);
                let origin_tags = origin_result.cache_tags(priority_cache_tags);

                if origin_result.should_purge_related() {
                    let ray_id = request_context.cloudflare_request_context.ray_id.to_string();
                    let global_cache = self.global_cache.clone();
                    let purge_cache_tags = origin_tags.clone();

                    log::info!(ray_id, "Purging global cache by tags: {:?}", purge_cache_tags);

                    request_context
                        .wait_until_promises
                        .borrow_mut() // safe due to the single threaded runtime
                        .push(
                            SendWrapper::new(async move {
                                if let Err(err) = global_cache
                                    .purge_by_tags(purge_cache_tags)
                                    .instrument(tracing::info_span!("cache_tags_purge"))
                                    .await
                                {
                                    log::error!(ray_id, "Error global cache purge by tags: {err}");
                                }
                            })
                            .boxed(),
                        );
                }

                if origin_result.should_cache() && !request_context.cloudflare_request_context.cache_control.no_store {
                    let ray_id = request_context.cloudflare_request_context.ray_id.to_string();
                    let key = cache_key.to_string();
                    let cache_name = self.cache_name.to_string();
                    let put_value = origin_result.clone();

                    request_context
                        .wait_until_promises
                        .borrow_mut() // safe due to the single threaded runtime
                        .push(
                            SendWrapper::new(async move {
                                if let Err(err) = P::put(
                                    &cache_name,
                                    &ray_id,
                                    &key,
                                    CacheEntryState::Fresh,
                                    put_value,
                                    origin_tags,
                                )
                                .instrument(tracing::info_span!("cache_put"))
                                .await
                                {
                                    log::error!(ray_id, "Error cache PUT: {err}");
                                }
                            })
                            .boxed(),
                        );

                    return Ok(CacheResponse::Miss(origin_result));
                }

                Ok(CacheResponse::Bypass(origin_result))
            }
        }
    }

    async fn update_stale(
        &self,
        request_context: &RequestContext,
        key: &str,
        existing_value: Arc<CV>,
        source_future: impl Future<Output = worker::Result<CV>> + 'static,
        priority_tags: Vec<String>,
    ) -> CacheResult<bool> {
        let key = key.to_string();
        let cache_name = self.cache_name.to_string();
        let ray_id = request_context.cloudflare_request_context.ray_id.to_string();
        let existing_tags = existing_value.cache_tags(priority_tags.clone());

        // refresh the cache async and update the existing entry state
        request_context.wait_until_promises.borrow_mut().push(
            SendWrapper::new(async move {
                let put_futures = P::put(
                    &cache_name,
                    &ray_id,
                    &key,
                    CacheEntryState::Updating,
                    existing_value.clone(),
                    existing_tags.clone(),
                )
                .instrument(tracing::info_span!("cache_put_updating"))
                .inspect_err(|err| {
                    log::error!(
                        ray_id,
                        "Error transitioning cache entry to {} - {err}",
                        CacheEntryState::Updating
                    );
                });

                let (_, source_result) = futures_util::join!(put_futures, source_future);

                match source_result {
                    Ok(fresh_value) => {
                        log::info!(ray_id, "Successfully fetched new value for cache from origin");
                        let fresh_cache_tags = fresh_value.cache_tags(priority_tags);

                        let _ = P::put(
                            &cache_name,
                            &ray_id,
                            &key,
                            CacheEntryState::Fresh,
                            Arc::new(fresh_value),
                            fresh_cache_tags,
                        )
                        .instrument(tracing::info_span!("cache_put_refresh"))
                        .inspect_err(|err| {
                            // if this errors we're probably stuck in `UPDATING` state or
                            // we're serving a cache entries as `FRESH` where in reality they are stale (in case the `UPDATING` transition failed)
                            log::error!(ray_id, "Error updating stale cache entry with fresh value - {err}");
                        })
                        .await;
                    }
                    Err(err) => {
                        log::error!(ray_id, "Error fetching fresh value for a stale cache entry: {err}");
                        let _ = P::put(
                            &cache_name,
                            &ray_id,
                            &key,
                            CacheEntryState::Stale,
                            existing_value,
                            existing_tags,
                        )
                        .instrument(tracing::info_span!("cache_put_stale"))
                        .inspect_err(|err| {
                            // if this errors we're probably stuck in `UPDATING` state or
                            // we're serving cache entries as `FRESH` when in reality they are stale (in case the `UPDATING` transition failed)
                            log::error!(
                                ray_id,
                                "Error transitioning cache entry to {} - {err}",
                                CacheEntryState::Stale
                            );
                        })
                        .await;
                    }
                };
            })
            .boxed(),
        );

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::gcache::CacheResponse;
    use crate::cache::{
        Cache, CacheEntryState, CacheProvider, CacheProviderResponse, CacheResult, Cacheable, GlobalCacheProvider,
    };
    use crate::platform::config::Config;
    use crate::platform::context::RequestContext;
    use dynaql::parser::types::OperationType;
    use futures_util::future::BoxFuture;
    use gateway_protocol::CustomerDeploymentConfig;
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::hash::Hash;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, RwLock};
    use worker_utils::CloudflareRequestContext;

    struct DefaultTestGlobalCache;
    impl GlobalCacheProvider for DefaultTestGlobalCache {}

    #[derive(serde::Deserialize, serde::Serialize, Default, Clone, Eq, PartialEq, Debug, Hash)]
    struct TestCacheValue {
        max_age_seconds: usize,
        stale_seconds: usize,
        ttl_seconds: usize,
        operation_type: OperationType,
        tags: Vec<String>,
    }

    impl TestCacheValue {
        fn builder(n: usize, operation_type: OperationType, tags: Vec<String>) -> Self {
            Self {
                max_age_seconds: n,
                stale_seconds: n,
                ttl_seconds: n,
                operation_type,
                tags,
            }
        }
    }

    impl Cacheable for TestCacheValue {
        fn max_age_seconds(&self) -> usize {
            self.max_age_seconds
        }

        fn stale_seconds(&self) -> usize {
            self.stale_seconds
        }

        fn ttl_seconds(&self) -> usize {
            self.max_age_seconds + self.stale_seconds
        }

        fn cache_tags(&self, priority_tags: Vec<String>) -> Vec<String> {
            let mut cache_tags = Vec::with_capacity(self.tags.len() + priority_tags.len());

            cache_tags.extend(priority_tags);
            cache_tags.extend(self.tags.clone());

            cache_tags
        }

        fn should_purge_related(&self) -> bool {
            self.operation_type == OperationType::Mutation && !self.tags.is_empty()
        }

        fn should_cache(&self) -> bool {
            self.operation_type != OperationType::Mutation
        }
    }

    fn build_request_context(
        config: Config,
        wait_until_promises: Arc<RefCell<Vec<BoxFuture<'static, ()>>>>,
    ) -> RequestContext {
        RequestContext {
            #[cfg(not(feature = "local"))]
            api_key_auth: crate::auth::ApiKeyAuth::default(),
            cloudflare_request_context: CloudflareRequestContext::default(),
            closest_aws_region: rusoto_core::Region::EuNorth1,
            config: Arc::new(config),
            wait_until_promises,
        }
    }

    #[tokio::test]
    async fn should_successfully_get_miss() -> Result<(), &'static str> {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);
        static PUT_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait(? Send)]
        impl CacheProvider for TestCache {
            type Value = TestCacheValue;

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheProviderResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheProviderResponse::Miss)
            }

            async fn put(
                _cache_name: &str,
                _ray_id: &str,
                _key: &str,
                _status: CacheEntryState,
                _value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> CacheResult<()> {
                PUT_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let config = Config::default();
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string(), Box::new(DefaultTestGlobalCache));

        // act
        let cache_result = g_cache
            .cached(&request_context, "key", async {
                Ok(TestCacheValue::builder(2, OperationType::Query, vec![]))
            })
            .await;
        let futures = wait_until_promises
            .borrow_mut()
            .drain(..)
            .collect::<Vec<BoxFuture<'static, ()>>>();
        futures_util::future::join_all(futures).await;

        // assert
        assert!(cache_result.is_ok());
        let cache_response = cache_result.unwrap();

        match cache_response {
            CacheResponse::Miss(cache_value) => {
                assert_eq!(
                    cache_value.as_ref(),
                    &TestCacheValue::builder(2, OperationType::Query, vec![])
                );
                assert_eq!(1, PUT_CALLS.load(Ordering::SeqCst));
                assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
            }
            _ => return Err("should be a CacheResponse::Miss"),
        };

        Ok(())
    }

    #[tokio::test]
    async fn should_successfully_get_hit() -> Result<(), &'static str> {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);
        static PUT_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait(? Send)]
        impl CacheProvider for TestCache {
            type Value = TestCacheValue;

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheProviderResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheProviderResponse::Hit(TestCacheValue::builder(
                    1,
                    OperationType::Query,
                    vec![],
                )))
            }
        }

        let config = Config::default();
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string(), Box::new(DefaultTestGlobalCache));

        // act
        let cache_result = g_cache
            .cached(&request_context, "key", async {
                Ok(TestCacheValue::builder(2, OperationType::Query, vec![]))
            })
            .await;
        let futures = wait_until_promises
            .borrow_mut()
            .drain(..)
            .collect::<Vec<BoxFuture<'static, ()>>>();
        futures_util::future::join_all(futures).await;

        // assert
        assert!(cache_result.is_ok());
        let cache_response = cache_result.unwrap();

        match cache_response {
            CacheResponse::Hit(cache_value) => {
                assert_eq!(
                    cache_value.as_ref(),
                    &TestCacheValue::builder(1, OperationType::Query, vec![])
                );
                assert_eq!(0, PUT_CALLS.load(Ordering::SeqCst));
                assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
            }
            _ => return Err("should be a CacheResponse::Hit"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn should_successfully_update_stale() -> Result<(), &'static str> {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);
        static PUT_CALLS: AtomicUsize = AtomicUsize::new(0);
        static CACHE_ENTRY_STATES: RwLock<Vec<CacheEntryState>> = RwLock::new(Vec::new());
        static CACHE_ENTRY_VALUES: RwLock<Vec<Arc<TestCacheValue>>> = RwLock::new(Vec::new());

        struct TestCache;
        #[async_trait::async_trait(? Send)]
        impl CacheProvider for TestCache {
            type Value = TestCacheValue;

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheProviderResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheProviderResponse::Stale {
                    response: TestCacheValue::builder(1, OperationType::Query, vec![]),
                    is_updating: false,
                })
            }

            async fn put(
                _cache_name: &str,
                _ray_id: &str,
                _key: &str,
                status: CacheEntryState,
                value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> CacheResult<()> {
                PUT_CALLS.fetch_add(1, Ordering::SeqCst);
                CACHE_ENTRY_STATES.write().unwrap().push(status);
                CACHE_ENTRY_VALUES.write().unwrap().push(value);
                Ok(())
            }
        }

        let config = Config::default();
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string(), Box::new(DefaultTestGlobalCache));

        // act
        let cache_result = g_cache
            .cached(&request_context, "key", async {
                Ok(TestCacheValue::builder(2, OperationType::Query, vec![]))
            })
            .await;
        let futures = wait_until_promises
            .borrow_mut()
            .drain(..)
            .collect::<Vec<BoxFuture<'static, ()>>>();
        futures_util::future::join_all(futures).await;

        // assert
        assert!(cache_result.is_ok());
        let cache_response = cache_result.unwrap();

        match cache_response {
            CacheResponse::Stale { response, is_updating } => {
                assert_eq!(
                    response.as_ref(),
                    &TestCacheValue::builder(1, OperationType::Query, vec![])
                );
                assert!(is_updating);
            }
            _ => return Err("should be a CacheResponse::Stale"),
        }

        // assert final state
        assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
        assert_eq!(2, PUT_CALLS.load(Ordering::SeqCst));

        let cache_entry_states = CACHE_ENTRY_STATES.read().unwrap();
        let cache_entry_states: HashSet<&CacheEntryState> = cache_entry_states.iter().collect();
        let expected_states: HashSet<&CacheEntryState> =
            HashSet::from_iter(&[CacheEntryState::Updating, CacheEntryState::Fresh]);
        assert_eq!(expected_states, cache_entry_states);

        let cache_entry_values = CACHE_ENTRY_VALUES.read().unwrap();
        let expected_cache_values = vec![
            TestCacheValue::builder(1, OperationType::Query, vec![]),
            TestCacheValue::builder(2, OperationType::Query, vec![]),
        ];

        let cache_entry_values: HashSet<&TestCacheValue> = cache_entry_values.iter().map(|v| v.as_ref()).collect();
        let expected_cache_values: HashSet<&TestCacheValue> = expected_cache_values.iter().collect();
        assert_eq!(expected_cache_values, cache_entry_values);

        Ok(())
    }

    #[tokio::test]
    async fn should_fail_update_stale() -> Result<(), &'static str> {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);
        static PUT_CALLS: AtomicUsize = AtomicUsize::new(0);
        static CACHE_ENTRY_STATES: RwLock<Vec<CacheEntryState>> = RwLock::new(Vec::new());
        static CACHE_ENTRY_VALUES: RwLock<Vec<Arc<TestCacheValue>>> = RwLock::new(Vec::new());
        static CACHE_ENTRY_STALE: AtomicBool = AtomicBool::new(false);

        struct TestCache;
        #[async_trait::async_trait(? Send)]
        impl CacheProvider for TestCache {
            type Value = TestCacheValue;

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheProviderResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheProviderResponse::Stale {
                    response: TestCacheValue::builder(1, OperationType::Query, vec![]),
                    is_updating: false,
                })
            }

            async fn put(
                _cache_name: &str,
                _ray_id: &str,
                _key: &str,
                status: CacheEntryState,
                value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> CacheResult<()> {
                PUT_CALLS.fetch_add(1, Ordering::SeqCst);
                CACHE_ENTRY_STALE.swap(matches!(status, CacheEntryState::Stale), Ordering::SeqCst);
                CACHE_ENTRY_STATES.write().unwrap().push(status);
                CACHE_ENTRY_VALUES.write().unwrap().push(value);
                Ok(())
            }
        }

        let config = Config::default();
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string(), Box::new(DefaultTestGlobalCache));

        // act
        let cache_result = g_cache
            .cached(&request_context, "key", async {
                Err(worker::Error::RustError("failed_source".to_string()))
            })
            .await;

        let futures = wait_until_promises
            .borrow_mut()
            .drain(..)
            .collect::<Vec<BoxFuture<'static, ()>>>();
        futures_util::future::join_all(futures).await;

        // assert
        assert!(cache_result.is_ok());
        let cache_response = cache_result.unwrap();

        match cache_response {
            CacheResponse::Stale { response, is_updating } => {
                assert_eq!(
                    response.as_ref(),
                    &TestCacheValue::builder(1, OperationType::Query, vec![])
                );
                assert!(is_updating);
            }
            _ => return Err("should be a CacheResponse::Stale"),
        }

        // assert final state
        assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
        assert_eq!(2, PUT_CALLS.load(Ordering::SeqCst));
        assert!(CACHE_ENTRY_STALE.load(Ordering::SeqCst));

        let cache_entry_states = CACHE_ENTRY_STATES.read().unwrap();
        let cache_entry_states: HashSet<&CacheEntryState> = cache_entry_states.iter().collect();
        let expected_states: HashSet<&CacheEntryState> =
            HashSet::from_iter(&[CacheEntryState::Updating, CacheEntryState::Stale]);
        assert_eq!(expected_states, cache_entry_states);

        let cache_entry_values = CACHE_ENTRY_VALUES.read().unwrap();
        let expected_cache_values = vec![
            TestCacheValue::builder(1, OperationType::Query, vec![]),
            TestCacheValue::builder(1, OperationType::Query, vec![]),
        ];

        let cache_entry_values: HashSet<&TestCacheValue> = cache_entry_values.iter().map(|v| v.as_ref()).collect();
        let expected_cache_values: HashSet<&TestCacheValue> = expected_cache_values.iter().collect();
        assert_eq!(expected_cache_values, cache_entry_values);

        Ok(())
    }

    #[tokio::test]
    async fn should_not_update_stale_entry_when_updating() -> Result<(), &'static str> {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait(? Send)]
        impl CacheProvider for TestCache {
            type Value = TestCacheValue;

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheProviderResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheProviderResponse::Stale {
                    response: TestCacheValue::builder(1, OperationType::Query, vec![]),
                    is_updating: true,
                })
            }
        }

        let config = Config::default();
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string(), Box::new(DefaultTestGlobalCache));

        // act
        let cache_result = g_cache
            .cached(&request_context, "key", async {
                Err(worker::Error::RustError("failed_source".to_string()))
            })
            .await;

        let futures = wait_until_promises
            .borrow_mut()
            .drain(..)
            .collect::<Vec<BoxFuture<'static, ()>>>();
        futures_util::future::join_all(futures).await;

        // assert
        assert!(cache_result.is_ok());
        let cache_response = cache_result.unwrap();

        match cache_response {
            CacheResponse::Stale { response, is_updating } => {
                assert_eq!(
                    response.as_ref(),
                    &TestCacheValue::builder(1, OperationType::Query, vec![])
                );
                assert!(is_updating);
                assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
            }
            _ => return Err("should be a CacheResponse::Stale"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn should_successfully_get_bypass() -> Result<(), &'static str> {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);
        static PUT_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait(? Send)]
        impl CacheProvider for TestCache {
            type Value = TestCacheValue;

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheProviderResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheProviderResponse::Miss)
            }

            async fn put(
                _cache_name: &str,
                _ray_id: &str,
                _key: &str,
                _status: CacheEntryState,
                _value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> CacheResult<()> {
                PUT_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let config = Config::default();
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string(), Box::new(DefaultTestGlobalCache));

        // act
        let cache_result = g_cache
            .cached(&request_context, "key", async {
                Ok(TestCacheValue::builder(2, OperationType::Mutation, vec![]))
            })
            .await;
        let futures = wait_until_promises
            .borrow_mut()
            .drain(..)
            .collect::<Vec<BoxFuture<'static, ()>>>();
        futures_util::future::join_all(futures).await;

        // assert
        assert!(cache_result.is_ok());
        let cache_response = cache_result.unwrap();

        match cache_response {
            CacheResponse::Bypass(cache_value) => {
                assert_eq!(
                    cache_value.as_ref(),
                    &TestCacheValue::builder(2, OperationType::Mutation, vec![])
                );
                assert_eq!(0, PUT_CALLS.load(Ordering::SeqCst));
                assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
            }
            _ => return Err("should be a CacheResponse::Bypass"),
        };

        Ok(())
    }

    #[tokio::test]
    async fn should_successfully_purge_global() -> Result<(), &'static str> {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);
        static PUT_CALLS: AtomicUsize = AtomicUsize::new(0);
        static PURGE_CALLS: AtomicUsize = AtomicUsize::new(0);
        static PURGE_TAGS: RwLock<Vec<String>> = RwLock::new(Vec::new());

        struct TestCache;
        #[async_trait::async_trait(? Send)]
        impl CacheProvider for TestCache {
            type Value = TestCacheValue;

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheProviderResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheProviderResponse::Miss)
            }

            async fn put(
                _cache_name: &str,
                _ray_id: &str,
                _key: &str,
                _status: CacheEntryState,
                _value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> CacheResult<()> {
                PUT_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        struct PurgeTestGlobalCache;
        #[async_trait::async_trait(?Send)]
        impl GlobalCacheProvider for PurgeTestGlobalCache {
            async fn purge_by_tags(&self, tags: Vec<String>) -> CacheResult<()> {
                PURGE_CALLS.fetch_add(1, Ordering::SeqCst);
                PURGE_TAGS.write().unwrap().extend(tags);
                Ok(())
            }
        }

        let config = Config {
            customer_deployment_config: CustomerDeploymentConfig {
                project_id: "project_id".to_string(),
                github_ref_name: Some("github_ref_name".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string(), Box::new(PurgeTestGlobalCache));

        // act
        let cache_result = g_cache
            .cached(&request_context, "key", async {
                Ok(TestCacheValue::builder(
                    2,
                    OperationType::Mutation,
                    vec!["tag".to_string()],
                ))
            })
            .await;
        let futures = wait_until_promises
            .borrow_mut()
            .drain(..)
            .collect::<Vec<BoxFuture<'static, ()>>>();
        futures_util::future::join_all(futures).await;

        // assert
        assert!(cache_result.is_ok());
        let cache_response = cache_result.unwrap();

        match cache_response {
            CacheResponse::Bypass(cache_value) => {
                assert_eq!(
                    cache_value.as_ref(),
                    &TestCacheValue::builder(2, OperationType::Mutation, vec!["tag".to_string()])
                );
                assert_eq!(0, PUT_CALLS.load(Ordering::SeqCst));
                assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
                assert_eq!(1, PURGE_CALLS.load(Ordering::SeqCst));

                // order is important because the edge cache implementation caps at 16KB
                // we want to make sure project_id and branch are included
                let expected_purge_tags = vec![
                    "github_ref_name".to_string(),
                    "project_id".to_string(),
                    "tag".to_string(),
                ];
                let purge_tags = PURGE_TAGS
                    .read()
                    .unwrap()
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>();
                assert_eq!(expected_purge_tags, purge_tags);
            }
            _ => return Err("should be a CacheResponse::Bypass"),
        };

        Ok(())
    }
}
