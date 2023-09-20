use std::{future::Future, sync::Arc, time::Duration};

use common_types::auth::ExecutionAuth;
use futures_util::{future::BoxFuture, FutureExt, TryFutureExt};
use http::status::StatusCode;
use runtime_ext::cache::{Cache, Cacheable, Entry, EntryState};
use send_wrapper::SendWrapper;
use tracing::{info_span, Instrument};

mod build_key;
mod key;

pub const X_GRAFBASE_CACHE: &str = "x-grafbase-cache";
pub use build_key::{build_cache_key, BuildKeyError};

#[derive(thiserror::Error, Debug)]
pub enum ServerCacheError<ValueError> {
    #[error(transparent)]
    Cache(#[from] runtime_ext::cache::Error),
    #[error(transparent)]
    Value(ValueError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CacheReadStatus {
    Hit,
    Bypass,
    Miss { max_age: Duration },
    Stale { revalidated: bool },
}

impl ToString for CacheReadStatus {
    fn to_string(&self) -> String {
        match self {
            CacheReadStatus::Hit => "HIT".to_string(),
            CacheReadStatus::Miss { .. } => "MISS".to_string(),
            CacheReadStatus::Stale { revalidated } => {
                if *revalidated {
                    "UPDATING".to_string()
                } else {
                    "STALE".to_string()
                }
            }
            CacheReadStatus::Bypass => "BYPASS".to_string(),
        }
    }
}

impl CacheReadStatus {
    fn into_headers(self) -> Vec<(String, String)> {
        let mut headers = vec![(X_GRAFBASE_CACHE.to_string(), self.to_string())];
        if let CacheReadStatus::Miss { max_age } = self {
            headers.push((
                http::header::CACHE_CONTROL.to_string(),
                format!("public, max-age: {}", max_age.as_secs()),
            ));
        }
        headers
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionResponse<T> {
    Stale {
        response: T,
        cache_revalidation: bool,
    },
    Cached(T),
    Origin {
        response: T,
        cache_read: Option<CacheReadStatus>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum CacheResponse<T> {
    Hit(T),
    Miss(T),
    Bypass(T),
    Stale { response: T, updating: bool },
}

impl<T: Cacheable> From<CacheResponse<Arc<T>>> for ExecutionResponse<Arc<T>> {
    fn from(value: CacheResponse<Arc<T>>) -> Self {
        match value {
            CacheResponse::Hit(response) => ExecutionResponse::Cached(response),
            CacheResponse::Stale {
                response,
                updating: revalidated,
            } => ExecutionResponse::Stale {
                response,
                cache_revalidation: revalidated,
            },
            CacheResponse::Miss(response) => {
                let max_age = response.max_age_seconds() as u64;
                ExecutionResponse::Origin {
                    response,
                    cache_read: Some(CacheReadStatus::Miss {
                        max_age: Duration::from_secs(max_age),
                    }),
                }
            }
            CacheResponse::Bypass(response) => ExecutionResponse::Origin {
                response,
                cache_read: Some(CacheReadStatus::Bypass),
            },
        }
    }
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

pub trait CacheContext {
    fn ray_id(&self) -> &str;
    fn wait_until_push(&self, fut: BoxFuture<'static, ()>);

    fn header(&self, name: &str) -> Option<String>;
    fn namespace(&self) -> String;
    fn common_cache_tags(&self) -> Vec<String>;
    fn control(&self) -> CacheControl;
    fn cache_config(&self) -> &engine::registry::CacheConfig;
    fn subdomain(&self) -> &str;
    fn host_name(&self) -> &str;
    fn caching_enabled(&self) -> bool;
}

pub fn process_execution_response<Context, Response>(
    ctx: &Context,
    response: Result<ExecutionResponse<Arc<engine::Response>>, ServerCacheError<gateway_adapter::ExecutionError>>,
) -> Result<Response, crate::ServerError>
where
    Context: CacheContext,
    Response: crate::Response<Context = Context>,
{
    let (response, headers) = match response {
        Ok(execution_response) => match execution_response {
            ExecutionResponse::Cached(cached) => (cached, CacheReadStatus::Hit.into_headers()),
            ExecutionResponse::Stale {
                response,
                cache_revalidation: revalidated,
            } => (response, CacheReadStatus::Stale { revalidated }.into_headers()),
            ExecutionResponse::Origin { response, cache_read } => {
                (response, cache_read.map(|c| c.into_headers()).unwrap_or_default())
            }
        },
        Err(e) => {
            log::error!(ctx.ray_id(), "Execution error: {}", e);
            return Response::error(ctx, "Execution error", StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    Response::engine(ctx, response).and_then(|resp| resp.with_headers(headers))
}

pub async fn execute_with_cache<Ctx, Value, ValueError, ValueFut, ValueFutBuilder>(
    cache: &Arc<impl Cache<Value = Value> + 'static>,
    ctx: &Ctx,
    request: engine::Request,
    auth: ExecutionAuth,
    callback: ValueFutBuilder,
) -> Result<ExecutionResponse<Arc<Value>>, ServerCacheError<ValueError>>
where
    Value: Cacheable + 'static,
    Ctx: CacheContext,
    ValueError: std::fmt::Display,
    ValueFut: Future<Output = Result<Arc<Value>, ValueError>> + 'static,
    ValueFutBuilder: FnOnce(&Ctx, engine::Request, ExecutionAuth) -> ValueFut,
{
    if ctx.caching_enabled() {
        match build_cache_key(ctx, &request, &auth) {
            Ok(cache_key) => {
                let cache_response = cached(cache, ctx, cache_key, callback(ctx, request, auth))
                    .instrument(info_span!("cached_execution"))
                    .await?;

                Ok(cache_response.into())
            }
            Err(err) => {
                log::debug!(ctx.ray_id(), "error creating cache key, bypassing cache: {err}",);

                Ok(ExecutionResponse::Origin {
                    response: callback(ctx, request, auth).await.map_err(ServerCacheError::Value)?,
                    cache_read: Some(CacheReadStatus::Bypass),
                })
            }
        }
    } else {
        Ok(ExecutionResponse::Origin {
            response: callback(ctx, request, auth).await.map_err(ServerCacheError::Value)?,
            cache_read: None,
        })
    }
}

pub(super) async fn cached<Value, ValueError, ValueFut>(
    cache: &Arc<impl Cache<Value = Value> + 'static>,
    ctx: &impl CacheContext,
    key: String,
    value_fut: ValueFut,
) -> Result<CacheResponse<Arc<Value>>, ServerCacheError<ValueError>>
where
    Value: Cacheable + 'static,
    ValueError: std::fmt::Display,
    ValueFut: Future<Output = Result<Arc<Value>, ValueError>> + 'static,
{
    let namespace = ctx.namespace();
    // skip if the incoming request doesn't want cached values, forces origin revalidation
    let cached_value = if ctx.control().no_cache {
        Entry::Miss
    } else {
        cache
            .get(&namespace, &key)
            .instrument(info_span!("cache_get"))
            .await
            .unwrap_or_else(|e| {
                log::warn!(ctx.ray_id(), "Error loading {key} from cache: {e}",);
                Entry::Miss
            })
    };

    let priority_cache_tags = ctx.common_cache_tags();

    match cached_value {
        Entry::Stale {
            response,
            state,
            is_early_stale,
        } => {
            let response_arc = Arc::new(response);
            let mut revalidated = false;

            // we only want to issue a revalidation if one is not already in progress
            if state != EntryState::UpdateInProgress {
                revalidated = true;

                update_stale(
                    cache,
                    ctx,
                    key.clone(),
                    response_arc.clone(),
                    value_fut,
                    priority_cache_tags,
                )
                .await;
            }

            log::info!(ctx.ray_id(), "Cache STALE - {revalidated} - {key}");

            // early stales mean early refreshes on our part
            // they shouldn't be considered as stale from a client perspective
            if is_early_stale {
                log::info!(ctx.ray_id(), "Cache HIT - {key}");
                return Ok(CacheResponse::Hit(response_arc));
            }

            Ok(CacheResponse::Stale {
                response: response_arc,
                updating: revalidated,
            })
        }
        Entry::Hit(gql_response) => {
            log::info!(ctx.ray_id(), "Cache HIT - {key}");

            Ok(CacheResponse::Hit(Arc::new(gql_response)))
        }
        Entry::Miss => {
            log::info!(ctx.ray_id(), "Cache MISS - {key}");

            let origin_result = value_fut.await.map_err(ServerCacheError::Value)?;
            let origin_tags = origin_result.cache_tags(priority_cache_tags);

            if origin_result.should_purge_related() {
                let ray_id = ctx.ray_id().to_string();
                let cache = Arc::clone(cache);
                let purge_cache_tags = origin_tags.clone();

                log::info!(ray_id, "Purging global cache by tags: {:?}", purge_cache_tags);

                ctx.wait_until_push(
                    SendWrapper::new(async move {
                        if let Err(err) = cache
                            .purge_by_tags(purge_cache_tags)
                            .instrument(info_span!("cache_tags_purge"))
                            .await
                        {
                            log::error!(ray_id, "Error global cache purge by tags: {err}");
                        }
                    })
                    .boxed(),
                );
            }

            if origin_result.should_cache() && !ctx.control().no_store {
                let ray_id = ctx.ray_id().to_string();
                let put_value = origin_result.clone();
                let cache = Arc::clone(cache);

                ctx.wait_until_push(
                    SendWrapper::new(async move {
                        if let Err(err) = cache
                            .put(&namespace, &ray_id, &key, EntryState::Fresh, put_value, origin_tags)
                            .instrument(info_span!("cache_put"))
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

async fn update_stale<Value, ValueError, ValueFut>(
    cache: &Arc<impl Cache<Value = Value> + 'static>,
    ctx: &impl CacheContext,
    key: String,
    existing_value: Arc<Value>,
    value_fut: ValueFut,
    priority_tags: Vec<String>,
) where
    Value: Cacheable + 'static,
    ValueError: std::fmt::Display,
    ValueFut: Future<Output = Result<Arc<Value>, ValueError>> + 'static,
{
    let ray_id = ctx.ray_id().to_string();
    let namespace = ctx.namespace();
    let key = key.to_string();
    let existing_tags = existing_value.cache_tags(priority_tags.clone());

    // refresh the cache async and update the existing entry state
    let cache = Arc::clone(cache);
    ctx.wait_until_push(
        SendWrapper::new(async move {
            let put_futures = cache
                .put(
                    &namespace,
                    &ray_id,
                    &key,
                    EntryState::UpdateInProgress,
                    existing_value.clone(),
                    existing_tags.clone(),
                )
                .instrument(info_span!("cache_put_updating"))
                .inspect_err(|err| {
                    log::error!(
                        ray_id,
                        "Error transitioning cache entry to {} - {err}",
                        EntryState::UpdateInProgress
                    );
                });

            let (_, source_result) = futures_util::join!(put_futures, value_fut);

            match source_result {
                Ok(fresh_value) => {
                    log::info!(ray_id, "Successfully fetched new value for cache from origin");
                    let fresh_cache_tags = fresh_value.cache_tags(priority_tags);

                    let _ = cache
                        .put(
                            &namespace,
                            &ray_id,
                            &key,
                            EntryState::Fresh,
                            fresh_value,
                            fresh_cache_tags,
                        )
                        .instrument(info_span!("cache_put_refresh"))
                        .inspect_err(|err| {
                            // if this errors we're probably stuck in `UPDATING` state or
                            // we're serving a cache entries as `FRESH` where in reality they are stale (in case the `UPDATING` transition failed)
                            log::error!(ray_id, "Error updating stale cache entry with fresh value - {err}");
                        })
                        .await;
                }
                Err(err) => {
                    log::error!(ray_id, "Error fetching fresh value for a stale cache entry: {err}");
                    let _ = cache
                        .put(
                            &namespace,
                            &ray_id,
                            &key,
                            EntryState::Stale,
                            existing_value,
                            existing_tags,
                        )
                        .instrument(info_span!("cache_put_stale"))
                        .inspect_err(|err| {
                            // if this errors we're probably stuck in `UPDATING` state or
                            // we're serving cache entries as `FRESH` when in reality they are stale (in case the `UPDATING` transition failed)
                            log::error!(
                                ray_id,
                                "Error transitioning cache entry to {} - {err}",
                                EntryState::Stale
                            );
                        })
                        .await;
                }
            };
        })
        .boxed(),
    );
}

#[cfg(test)]
mod tests {
    use crate::cache::key::{CacheAccess, CacheKey};

    use super::*;
    use engine::parser::types::OperationType;
    use engine::registry::{CacheAccessScope, MetaField, MetaFieldType, MetaType, ObjectType, Registry};
    use gateway_adapter_platform as _;
    use serde as _;
    use tokio::sync::RwLock;

    use std::cell::RefCell;
    use std::collections::hash_map::DefaultHasher;
    use std::collections::{BTreeMap, BTreeSet, HashSet};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    use common_types::auth::ExecutionAuth;
    use engine::Request;
    use futures_util::future::BoxFuture;
    use runtime_ext::cache;
    use runtime_ext::cache::test_utils::FakeCache;

    const TEST: &str = "Test";
    const QUERY: &str = "query { Test { id } }";

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
                operation_type: OperationType::Query,
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
            [priority_tags, self.tags.clone()].into_iter().flatten().collect()
        }

        fn should_purge_related(&self) -> bool {
            self.operation_type == OperationType::Mutation && !self.tags.is_empty()
        }

        fn should_cache(&self) -> bool {
            self.operation_type != OperationType::Mutation
        }
    }

    #[derive(Default)]
    struct FakeCacheContext {
        cache_config: engine::registry::CacheConfig,
        namespace: &'static str,
        subdomain: &'static str,
        common_cache_tags: Vec<&'static str>,
        caching_enabled: bool,
        cache_control: CacheControl,
        headers: Vec<(&'static str, &'static str)>,
        futures: RefCell<Vec<BoxFuture<'static, ()>>>,
    }

    impl FakeCacheContext {
        async fn wait_for_futures(&self) {
            let futures = self
                .futures
                .borrow_mut()
                .drain(..)
                .collect::<Vec<BoxFuture<'static, ()>>>();
            futures_util::future::join_all(futures).await;
        }
    }

    impl CacheContext for FakeCacheContext {
        fn ray_id(&self) -> &str {
            "ray-id"
        }

        fn wait_until_push(&self, fut: BoxFuture<'static, ()>) {
            self.futures.borrow_mut().push(fut);
        }

        fn header(&self, name: &str) -> Option<String> {
            self.headers
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| (*v).to_string())
        }

        fn namespace(&self) -> String {
            self.namespace.to_string()
        }

        fn common_cache_tags(&self) -> Vec<String> {
            self.common_cache_tags.iter().map(|s| (*s).to_string()).collect()
        }

        fn control(&self) -> CacheControl {
            self.cache_control.clone()
        }

        fn cache_config(&self) -> &engine::registry::CacheConfig {
            &self.cache_config
        }

        fn subdomain(&self) -> &str {
            self.subdomain
        }

        fn caching_enabled(&self) -> bool {
            self.caching_enabled
        }

        fn host_name(&self) -> &str {
            ""
        }
    }

    #[tokio::test]
    async fn should_execute_without_cache() {
        struct TestCache;
        impl FakeCache for TestCache {
            type Value = Dummy;
        }

        let res = dummy("Hello world!");
        let res2 = Arc::clone(&res);
        let response = execute_with_cache(
            &Arc::new(TestCache),
            &FakeCacheContext { ..Default::default() },
            Request::new("query test { test }"),
            ExecutionAuth::ApiKey,
            |_, _, _| async { anyhow::Ok(res2) },
        )
        .await
        .unwrap();

        assert_eq!(
            response,
            ExecutionResponse::Origin {
                response: res,
                cache_read: None
            }
        );
    }

    #[tokio::test]
    async fn should_execute_with_cache_miss() {
        struct TestCache;
        #[async_trait::async_trait(?Send)]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _namespace: &str, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                Ok(cache::Entry::Miss)
            }

            async fn put(
                &self,
                _namespace: &str,
                _ray_id: &str,
                _key: &str,
                _state: cache::EntryState,
                _value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> cache::Result<()> {
                Ok(())
            }
        }

        let res = dummy("Hi!");
        let res2 = res.clone();
        let response = execute_with_cache(
            &Arc::new(TestCache),
            &FakeCacheContext {
                caching_enabled: true,
                cache_config: build_cache_config(None),
                ..Default::default()
            },
            Request::new(QUERY),
            ExecutionAuth::ApiKey,
            |_, _, _| async { anyhow::Ok(res2) },
        )
        .await
        .unwrap();

        assert_eq!(
            response,
            ExecutionResponse::Origin {
                response: res,
                cache_read: Some(CacheReadStatus::Miss {
                    max_age: Duration::from_secs(1),
                })
            }
        );
    }

    #[tokio::test]
    async fn should_build_cache_key_for_auth() {
        // expected cache key
        let engine_request = Request::new(QUERY);
        let expected_cache_key =
            CacheKey::<DefaultHasher>::new(CacheAccess::Default(&ExecutionAuth::ApiKey), &engine_request, TEST);
        let expected_cache_key = format!("https://{}/{}", TEST, expected_cache_key.to_hash_string());

        expect_cache_key(
            FakeCacheContext {
                caching_enabled: true,
                subdomain: TEST,
                cache_config: build_cache_config(None),
                ..Default::default()
            },
            ExecutionAuth::ApiKey,
            expected_cache_key,
        )
        .await;
    }

    #[tokio::test]
    async fn should_build_cache_key_for_scoped_access_api_key() {
        // expected cache key
        let engine_request = Request::new(QUERY);
        let expected_cache_key = CacheKey::<DefaultHasher>::new(
            CacheAccess::Scoped(BTreeSet::from([ExecutionAuth::ApiKey.global_ops().to_string()])),
            &engine_request,
            TEST,
        );
        let expected_cache_key = format!("https://{}/{}", TEST, expected_cache_key.to_hash_string());

        expect_cache_key(
            FakeCacheContext {
                caching_enabled: true,
                subdomain: TEST,
                cache_config: build_cache_config(Some(engine::CacheControl {
                    access_scopes: Some([CacheAccessScope::ApiKey].into()),
                    ..Default::default()
                })),
                ..Default::default()
            },
            ExecutionAuth::ApiKey,
            expected_cache_key,
        )
        .await;
    }

    #[tokio::test]
    async fn should_build_cache_key_for_scoped_access_jwt() {
        // auth jwt
        let claim_value = serde_json::Value::String(TEST.to_string());
        let auth = ExecutionAuth::new_from_token(
            Default::default(),
            Default::default(),
            Default::default(),
            BTreeMap::from_iter([(TEST.to_string(), claim_value.clone())]),
        );

        // expected cache key
        let engine_request = Request::new(QUERY);
        let expected_cache_key = CacheKey::<DefaultHasher>::new(
            CacheAccess::Scoped(BTreeSet::from([claim_value.to_string()])),
            &engine_request,
            TEST,
        );
        let expected_cache_key = format!("https://{}/{}", TEST, expected_cache_key.to_hash_string());

        expect_cache_key(
            FakeCacheContext {
                caching_enabled: true,
                subdomain: TEST,
                cache_config: build_cache_config(Some(engine::CacheControl {
                    access_scopes: Some(
                        [CacheAccessScope::Jwt {
                            claim: TEST.to_string(),
                        }]
                        .into(),
                    ),
                    ..Default::default()
                })),
                ..Default::default()
            },
            auth,
            expected_cache_key,
        )
        .await;
    }

    #[tokio::test]
    async fn should_build_cache_key_for_scoped_access_header() {
        // expected cache key
        let engine_request = Request::new(QUERY);
        let expected_cache_key = CacheKey::<DefaultHasher>::new(
            CacheAccess::Scoped(BTreeSet::from([TEST.to_string()])),
            &engine_request,
            TEST,
        );
        let expected_cache_key = format!("https://{}/{}", TEST, expected_cache_key.to_hash_string());

        expect_cache_key(
            FakeCacheContext {
                caching_enabled: true,
                subdomain: TEST,
                cache_config: build_cache_config(Some(engine::CacheControl {
                    access_scopes: Some(
                        [CacheAccessScope::Header {
                            header: TEST.to_string(),
                        }]
                        .into(),
                    ),
                    ..Default::default()
                })),
                headers: vec![(TEST, TEST)],
                ..Default::default()
            },
            ExecutionAuth::ApiKey,
            expected_cache_key,
        )
        .await;
    }

    #[tokio::test]
    async fn should_build_cache_key_for_scoped_access_public() {
        // expected cache key
        let engine_request = Request::new(QUERY);
        let expected_cache_key = CacheKey::<DefaultHasher>::new(
            CacheAccess::Scoped(BTreeSet::from([ExecutionAuth::Public {
                global_ops: Default::default(),
            }
            .global_ops()
            .to_string()])),
            &engine_request,
            TEST,
        );
        let expected_cache_key = format!("https://{}/{}", TEST, expected_cache_key.to_hash_string());

        expect_cache_key(
            FakeCacheContext {
                caching_enabled: true,
                subdomain: TEST,
                cache_config: build_cache_config(Some(engine::CacheControl {
                    access_scopes: Some([CacheAccessScope::Public].into()),
                    ..Default::default()
                })),
                ..Default::default()
            },
            ExecutionAuth::Public {
                global_ops: Default::default(),
            },
            expected_cache_key,
        )
        .await;
    }

    #[tokio::test]
    async fn should_bypass_cache_on_missing_data_for_access_scopes() {
        struct TestCache;
        impl FakeCache for TestCache {
            type Value = Dummy;
        }

        let res = dummy("Hello world!");
        let res2 = Arc::clone(&res);
        let response = execute_with_cache(
            &Arc::new(TestCache),
            &FakeCacheContext {
                caching_enabled: true,
                subdomain: TEST,
                cache_config: build_cache_config(Some(engine::CacheControl {
                    access_scopes: Some(
                        [CacheAccessScope::Header {
                            header: TEST.to_string(),
                        }]
                        .into(),
                    ),
                    ..Default::default()
                })),
                ..Default::default()
            },
            Request::new(QUERY),
            ExecutionAuth::ApiKey,
            |_, _, _| async { anyhow::Ok(res2) },
        )
        .await
        .unwrap();

        assert_eq!(
            response,
            ExecutionResponse::Origin {
                response: res,
                cache_read: Some(CacheReadStatus::Bypass)
            }
        );
    }

    async fn expect_cache_key(ctx: FakeCacheContext, auth: ExecutionAuth, expected: String) {
        #[derive(Default)]
        struct TestCache(tokio::sync::Mutex<Vec<String>>);

        #[async_trait::async_trait(?Send)]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _namespace: &str, key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                self.0.lock().await.push(key.to_string());
                Ok(cache::Entry::Hit(Dummy::default()))
            }

            async fn put(
                &self,
                _namespace: &str,
                _ray_id: &str,
                _key: &str,
                _state: cache::EntryState,
                _value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> cache::Result<()> {
                Ok(())
            }
        }

        let cache = Arc::new(TestCache::default());
        let response = execute_with_cache(&cache, &ctx, Request::new(QUERY), auth, |_, _, _| async {
            anyhow::Ok(dummy(""))
        })
        .await
        .unwrap();

        assert_eq!(response, ExecutionResponse::Cached(dummy("")));
        assert_eq!(Arc::into_inner(cache).unwrap().0.into_inner(), vec![expected]);
    }

    fn build_cache_config(cache_control: Option<engine::CacheControl>) -> engine::registry::CacheConfig {
        let mut registry = Registry::new();
        registry.create_type(
            |_| {
                MetaType::Object({
                    let obj = ObjectType::new(TEST.to_string(), [MetaField::new("id", "String!")]);
                    if let Some(cache_control) = cache_control {
                        obj.with_cache_control(cache_control)
                    } else {
                        obj
                    }
                })
            },
            TEST,
            TEST,
        );

        registry.query_root_mut().fields_mut().unwrap().insert(
            TEST.to_string(),
            MetaField::new(TEST.to_string(), MetaFieldType::from(TEST)),
        );
        registry.into()
    }

    #[tokio::test]
    async fn should_successfully_get_miss() {
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);
        static PUT_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait(?Send)]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _namespace: &str, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(cache::Entry::Miss)
            }

            async fn put(
                &self,
                _namespace: &str,
                _ray_id: &str,
                _key: &str,
                _state: cache::EntryState,
                _value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> cache::Result<()> {
                PUT_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let dummy = Arc::new(Dummy::new(2, OperationType::Query, vec![]));
        let dummy2 = Arc::clone(&dummy);
        let ctx = FakeCacheContext::default();
        let response = cached(&Arc::new(TestCache), &ctx, "cache_key".to_string(), async {
            anyhow::Ok(dummy2)
        })
        .await
        .unwrap();
        ctx.wait_for_futures().await;
        assert_eq!(response, CacheResponse::Miss(dummy));
        assert_eq!(1, PUT_CALLS.load(Ordering::SeqCst));
        assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_successfully_get_hit() {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait(? Send)]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _namespace: &str, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(cache::Entry::Hit(Dummy {
                    value: "cached".to_string(),
                    ..Default::default()
                }))
            }
        }

        let ctx = FakeCacheContext::default();
        let response = cached(&Arc::new(TestCache), &ctx, "cache_key".to_string(), async {
            anyhow::Ok(Arc::new(Dummy {
                value: "new".to_string(),
                ..Default::default()
            }))
        })
        .await
        .unwrap();
        ctx.wait_for_futures().await;

        assert_eq!(response, CacheResponse::Hit(dummy("cached")));
        assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_successfully_update_stale() {
        #[derive(Default)]
        struct TestCache {
            get_calls: AtomicUsize,
            put_calls: tokio::sync::RwLock<Vec<(cache::EntryState, Arc<Dummy>)>>,
        }
        #[async_trait::async_trait(?Send)]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _namespace: &str, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                self.get_calls.fetch_add(1, Ordering::SeqCst);
                Ok(cache::Entry::Stale {
                    response: Dummy {
                        value: "stale".to_string(),
                        ..Default::default()
                    },
                    state: cache::EntryState::Fresh,
                    is_early_stale: false,
                })
            }

            async fn put(
                &self,
                _namespace: &str,
                _ray_id: &str,
                _key: &str,
                state: cache::EntryState,
                value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> cache::Result<()> {
                self.put_calls.write().await.push((state, value));
                Ok(())
            }
        }

        let ctx = FakeCacheContext::default();
        let cache = Arc::new(TestCache::default());
        let response = cached(&cache, &ctx, "cache_key".to_string(), async {
            anyhow::Ok(dummy("new"))
        })
        .await
        .unwrap();
        ctx.wait_for_futures().await;

        assert_eq!(
            response,
            CacheResponse::Stale {
                response: dummy("stale"),
                updating: true
            }
        );
        assert_eq!(1, cache.get_calls.load(Ordering::SeqCst));
        assert_eq!(
            cache.put_calls.read().await.iter().cloned().collect::<HashSet<_>>(),
            HashSet::from([
                (cache::EntryState::UpdateInProgress, dummy("stale")),
                (cache::EntryState::Fresh, dummy("new"))
            ])
        );
    }

    #[tokio::test]
    async fn should_fail_update_stale() {
        #[derive(Default)]
        struct TestCache {
            get_calls: AtomicUsize,
            entry_stale: AtomicBool,
            put_calls: tokio::sync::RwLock<Vec<(cache::EntryState, Arc<Dummy>)>>,
        }
        #[async_trait::async_trait(?Send)]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _namespace: &str, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                self.get_calls.fetch_add(1, Ordering::SeqCst);
                Ok(cache::Entry::Stale {
                    response: Dummy {
                        value: "stale".to_string(),
                        ..Default::default()
                    },
                    state: cache::EntryState::Fresh,
                    is_early_stale: false,
                })
            }

            async fn put(
                &self,
                _namespace: &str,
                _ray_id: &str,
                _key: &str,
                state: cache::EntryState,
                value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> cache::Result<()> {
                self.put_calls.write().await.push((state, value));
                self.entry_stale
                    .swap(matches!(state, cache::EntryState::Stale), Ordering::SeqCst);
                Ok(())
            }
        }

        let ctx = FakeCacheContext::default();
        let cache = Arc::new(TestCache::default());
        let response = cached(&cache, &ctx, "cache_key".to_string(), async {
            Err(anyhow::anyhow!("failed_source"))
        })
        .await
        .unwrap();
        ctx.wait_for_futures().await;

        assert_eq!(
            response,
            CacheResponse::Stale {
                response: dummy("stale"),
                updating: true
            }
        );
        assert_eq!(1, cache.get_calls.load(Ordering::SeqCst));
        assert!(cache.entry_stale.load(Ordering::SeqCst));
        assert_eq!(
            cache.put_calls.read().await.iter().cloned().collect::<HashSet<_>>(),
            HashSet::from([
                (cache::EntryState::UpdateInProgress, dummy("stale")),
                (cache::EntryState::Stale, dummy("stale"))
            ])
        );
    }

    #[tokio::test]
    async fn should_not_update_stale_entry_when_updating() {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait(? Send)]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _namespace: &str, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(cache::Entry::Stale {
                    response: Dummy {
                        value: "stale".to_string(),
                        ..Default::default()
                    },
                    state: cache::EntryState::UpdateInProgress,
                    is_early_stale: false,
                })
            }
        }

        let ctx = FakeCacheContext::default();
        let response = cached(&Arc::new(TestCache), &ctx, "cache_key".to_string(), async {
            Err(anyhow::anyhow!("failed_source"))
        })
        .await
        .unwrap();
        ctx.wait_for_futures().await;

        assert_eq!(
            response,
            CacheResponse::Stale {
                response: dummy("stale"),
                updating: false
            }
        );
        assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_successfully_get_bypass() {
        // prepare
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait(?Send)]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _namespace: &str, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(cache::Entry::Miss)
            }
        }

        let ctx = FakeCacheContext::default();
        let dummy = Arc::new(Dummy {
            value: "new".to_string(),
            operation_type: OperationType::Mutation,
            ..Default::default()
        });
        let dummy2 = Arc::clone(&dummy);
        let response = cached(&Arc::new(TestCache), &ctx, "cache_key".to_string(), async {
            anyhow::Ok(dummy2)
        })
        .await
        .unwrap();
        ctx.wait_for_futures().await;

        assert_eq!(response, CacheResponse::Bypass(dummy));
        assert_eq!(1, GET_CALLS.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn should_successfully_purge_global() {
        #[derive(Default)]
        struct TestCache {
            get_calls: AtomicUsize,
            purge_calls: RwLock<Vec<Vec<String>>>,
        }
        #[async_trait::async_trait(? Send)]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _namespace: &str, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                self.get_calls.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Miss)
            }

            async fn purge_by_tags(&self, tags: Vec<String>) -> cache::Result<()> {
                self.purge_calls.write().await.push(tags);
                Ok(())
            }
        }

        let ctx = FakeCacheContext {
            common_cache_tags: vec!["project"],
            ..Default::default()
        };
        let dummy = Arc::new(Dummy {
            value: "new".to_string(),
            operation_type: OperationType::Mutation,
            tags: vec!["tag".into()],
            ..Default::default()
        });
        let dummy2 = Arc::clone(&dummy);
        let cache = Arc::new(TestCache::default());
        let response = cached(&cache, &ctx, "cache_key".to_string(), async { anyhow::Ok(dummy2) })
            .await
            .unwrap();
        ctx.wait_for_futures().await;

        assert_eq!(response, CacheResponse::Bypass(dummy));
        assert_eq!(cache.get_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            cache.purge_calls.read().await.clone(),
            vec![vec!["project".to_string(), "tag".to_string()]]
        );
    }
}
