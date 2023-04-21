use futures_util::FutureExt;
use send_wrapper::SendWrapper;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use tracing_futures::Instrument;

use crate::cache::error::CacheError;
use crate::cache::{CacheEntryState, CacheProvider, CacheResponse, CacheResult, Cacheable};
use crate::platform::context::RequestContext;

pub struct Cache<CV, Provider> {
    cache_name: String,
    _cache_value: PhantomData<CV>,
    _cache_provider: PhantomData<Provider>,
}

impl<CV: Cacheable + 'static, P: CacheProvider<Value = CV>> Cache<CV, P> {
    pub fn new(cache_name: String) -> Self {
        Cache {
            cache_name,
            _cache_value: PhantomData,
            _cache_provider: PhantomData,
        }
    }

    pub async fn cached(
        &self,
        request_context: &RequestContext<'_>,
        cache_key: &str,
        source_future: impl Future<Output = worker::Result<CV>> + 'static,
    ) -> CacheResult<CacheResponse<Arc<CV>>> {
        let cached_value: CacheResponse<CV> = P::get(&self.cache_name, cache_key)
            .instrument(tracing::info_span!("cache_get"))
            .await?;

        match cached_value {
            CacheResponse::Stale {
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
                        .update_stale(request_context, cache_key, response_arc.clone(), source_future)
                        .await?;
                }

                Ok(CacheResponse::Stale {
                    response: response_arc,
                    is_updating,
                })
            }
            CacheResponse::Hit(gql_response) => {
                log::info!(
                    request_context.cloudflare_request_context.ray_id,
                    "Cache HIT - {cache_key}"
                );

                Ok(CacheResponse::Hit(Arc::new(gql_response)))
            }
            CacheResponse::Miss(_) => {
                log::info!(
                    request_context.cloudflare_request_context.ray_id,
                    "Cache MISS - {cache_key}"
                );

                let origin_result = Arc::new(source_future.await.map_err(CacheError::Origin)?);
                let key = cache_key.to_string();
                let cache_name = self.cache_name.to_string();
                let ray_id = request_context.cloudflare_request_context.ray_id.to_string();
                let put_value = origin_result.clone();

                request_context
                    .wait_until_promises
                    .borrow_mut() // safe due to the single threaded runtime
                    .push(
                        SendWrapper::new(async move {
                            if let Err(err) = P::put(&cache_name, &ray_id, &key, CacheEntryState::Fresh, put_value)
                                .instrument(tracing::info_span!("cache_put"))
                                .await
                            {
                                log::error!(ray_id, "Error cache PUT: {err}");
                            }
                        })
                        .boxed(),
                    );

                Ok(CacheResponse::Miss(origin_result))
            }
        }
    }

    #[allow(clippy::type_complexity)]
    async fn update_stale(
        &self,
        request_context: &RequestContext<'_>,
        key: &str,
        existing_value: Arc<CV>,
        source_future: impl Future<Output = worker::Result<CV>> + 'static,
    ) -> CacheResult<bool> {
        use futures_util::TryFutureExt;

        let key = key.to_string();
        let cache_name = self.cache_name.to_string();
        let ray_id = request_context.cloudflare_request_context.ray_id.to_string();

        // refresh the cache async and update the existing entry state
        request_context.wait_until_promises.borrow_mut().push(
            SendWrapper::new(async move {
                let put_futures = P::put(
                    &cache_name,
                    &ray_id,
                    &key,
                    CacheEntryState::Updating,
                    existing_value.clone(),
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
                        let _ = P::put(
                            &cache_name,
                            &ray_id,
                            &key,
                            CacheEntryState::Fresh,
                            Arc::new(fresh_value),
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

                        let _ = P::put(&cache_name, &ray_id, &key, CacheEntryState::Stale, existing_value)
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
    use crate::cache::{Cache, CacheEntryState, CacheProvider, CacheResponse, CacheResult, Cacheable};
    use crate::platform::config::Config;
    use crate::platform::context::RequestContext;
    use aws_region_nearby::AwsRegion;
    use futures_util::future::BoxFuture;
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, RwLock};
    use worker_utils::CloudflareRequestContext;

    #[derive(serde::Deserialize, serde::Serialize, Default, Clone, Eq, PartialEq, Debug, Hash)]
    struct TestCacheValue {
        max_age_seconds: usize,
        stale_seconds: usize,
        ttl_seconds: usize,
    }

    impl TestCacheValue {
        fn builder(n: usize) -> Self {
            Self {
                max_age_seconds: n,
                stale_seconds: n,
                ttl_seconds: n,
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
    }

    fn build_request_context(
        config: &'static Config,
        wait_until_promises: Arc<RefCell<Vec<BoxFuture<'static, ()>>>>,
    ) -> RequestContext<'static> {
        RequestContext {
            #[cfg(not(feature = "local"))]
            api_key_auth: crate::auth::ApiKeyAuth::default(),
            cloudflare_request_context: CloudflareRequestContext::default(),
            closest_aws_region: AwsRegion::EuNorth1,
            config,
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

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheResponse::Miss(TestCacheValue::default()))
            }

            async fn put(
                _cache_name: &str,
                _ray_id: &str,
                _key: &str,
                _status: CacheEntryState,
                _value: Arc<Self::Value>,
            ) -> CacheResult<()> {
                PUT_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let config = Box::leak(Box::default());
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string());

        // act
        let cache_result = g_cache
            .cached(&request_context, "key", async { Ok(TestCacheValue::builder(2)) })
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
                assert_eq!(cache_value.as_ref(), &TestCacheValue::builder(2));
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

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheResponse::Hit(TestCacheValue::builder(1)))
            }
        }

        let config = Box::leak(Box::default());
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string());

        // act
        let cache_result = g_cache
            .cached(&request_context, "key", async { Ok(TestCacheValue::builder(2)) })
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
                assert_eq!(cache_value.as_ref(), &TestCacheValue::builder(1));
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

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheResponse::Stale {
                    response: TestCacheValue::builder(1),
                    is_updating: false,
                })
            }

            async fn put(
                _cache_name: &str,
                _ray_id: &str,
                _key: &str,
                status: CacheEntryState,
                value: Arc<Self::Value>,
            ) -> CacheResult<()> {
                PUT_CALLS.fetch_add(1, Ordering::SeqCst);
                CACHE_ENTRY_STATES.write().unwrap().push(status);
                CACHE_ENTRY_VALUES.write().unwrap().push(value);
                Ok(())
            }
        }

        let config = Box::leak(Box::default());
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string());

        // act
        let cache_result = g_cache
            .cached(&request_context, "key", async { Ok(TestCacheValue::builder(2)) })
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
                assert_eq!(response.as_ref(), &TestCacheValue::builder(1));
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
        let expected_cache_values = vec![TestCacheValue::builder(1), TestCacheValue::builder(2)];

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

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheResponse::Stale {
                    response: TestCacheValue::builder(1),
                    is_updating: false,
                })
            }

            async fn put(
                _cache_name: &str,
                _ray_id: &str,
                _key: &str,
                status: CacheEntryState,
                value: Arc<Self::Value>,
            ) -> CacheResult<()> {
                PUT_CALLS.fetch_add(1, Ordering::SeqCst);
                CACHE_ENTRY_STALE.swap(matches!(status, CacheEntryState::Stale), Ordering::SeqCst);
                CACHE_ENTRY_STATES.write().unwrap().push(status);
                CACHE_ENTRY_VALUES.write().unwrap().push(value);
                Ok(())
            }
        }

        let config = Box::leak(Box::default());
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string());

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
                assert_eq!(response.as_ref(), &TestCacheValue::builder(1));
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
        let expected_cache_values = vec![TestCacheValue::builder(1), TestCacheValue::builder(1)];

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

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheResponse<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(CacheResponse::Stale {
                    response: TestCacheValue::builder(1),
                    is_updating: true,
                })
            }
        }

        let config = Box::leak(Box::default());
        let wait_until_promises = Arc::new(RefCell::new(Vec::new()));
        let request_context = build_request_context(config, wait_until_promises.clone());
        let g_cache = Cache::<TestCacheValue, TestCache>::new("test".to_string());

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
                assert_eq!(response.as_ref(), &TestCacheValue::builder(1));
                assert!(is_updating);
                assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
            }
            _ => return Err("should be a CacheResponse::Stale"),
        }

        Ok(())
    }
}
