use std::{borrow::Cow, future::Future, sync::Arc, time::Duration};

use async_runtime::make_send_on_wasm;
use common_types::auth::ExecutionAuth;
use futures_util::{FutureExt, TryFutureExt};
use http::status::StatusCode;

use runtime::cache::{Cache, Cacheable, Entry, EntryState};
use tracing::{info_span, Instrument};

mod build_key;
mod key;

pub const X_GRAFBASE_CACHE: &str = "x-grafbase-cache";
pub use build_key::{build_cache_key, BuildKeyError};

use engine::registry::CachePartialRegistry;

use super::RequestContext;

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

#[derive(Clone)]
pub struct CacheConfig<'a> {
    pub global_enabled: bool,
    pub subdomain: String,
    pub host_name: String,
    pub cache_control: CacheControl,
    pub partial_registry: Cow<'a, CachePartialRegistry>,
    pub common_cache_tags: Vec<String>,
}

pub fn process_execution_response<Context, Error, Response>(
    ctx: &Context,
    response: Result<ExecutionResponse<Arc<engine::Response>>, Error>,
) -> Result<Response, Error>
where
    Context: RequestContext,
    Error: std::fmt::Display,
    Response: super::Response<Context = Context, Error = Error>,
{
    let (response, headers) = match response {
        Ok(execution_response) => match execution_response {
            ExecutionResponse::Cached(cached) => (cached, CacheReadStatus::Hit.into_headers()),
            ExecutionResponse::Stale {
                response,
                cache_revalidation: revalidated,
            } => (response, CacheReadStatus::Stale { revalidated }.into_headers()),
            ExecutionResponse::Origin { response, cache_read } => (
                response,
                cache_read.map(CacheReadStatus::into_headers).unwrap_or_default(),
            ),
        },
        Err(e) => {
            log::error!(ctx.ray_id(), "Execution error: {}", e);
            return Response::error(ctx, "Execution error", StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    Response::engine(ctx, response).and_then(|resp| resp.with_headers(headers))
}

pub async fn execute_with_cache<Value, Error, ValueFut, ValueFutBuilder>(
    cache: &Arc<impl Cache<Value = Value> + 'static>,
    config: &CacheConfig<'_>,
    ctx: &impl RequestContext,
    request: engine::Request,
    auth: ExecutionAuth,
    callback: ValueFutBuilder,
) -> Result<ExecutionResponse<Arc<Value>>, Error>
where
    Value: Cacheable + 'static,
    Error: std::fmt::Display + Send,
    ValueFut: Future<Output = Result<Arc<Value>, Error>> + Send + 'static,
    ValueFutBuilder: FnOnce(engine::Request, ExecutionAuth) -> ValueFut,
{
    if config.global_enabled && config.partial_registry.enable_caching {
        match build_cache_key(config, ctx, &request, &auth) {
            Ok(cache_key) => {
                let cache_response = cached(cache, config, ctx, cache_key, callback(request, auth))
                    .instrument(info_span!("cached_execution"))
                    .await?;

                Ok(cache_response.into())
            }
            Err(err) => {
                log::debug!(ctx.ray_id(), "error creating cache key, bypassing cache: {err}",);

                Ok(ExecutionResponse::Origin {
                    response: callback(request, auth).await?,
                    cache_read: Some(CacheReadStatus::Bypass),
                })
            }
        }
    } else {
        Ok(ExecutionResponse::Origin {
            response: callback(request, auth).await?,
            cache_read: None,
        })
    }
}

pub(super) async fn cached<Value, Error, ValueFut>(
    cache: &Arc<impl Cache<Value = Value> + 'static>,
    config: &CacheConfig<'_>,
    ctx: &impl RequestContext,
    key: String,
    value_fut: ValueFut,
) -> Result<CacheResponse<Arc<Value>>, Error>
where
    Value: Cacheable + 'static,
    Error: std::fmt::Display + Send,
    ValueFut: Future<Output = Result<Arc<Value>, Error>> + Send + 'static,
{
    // skip if the incoming request doesn't want cached values, forces origin revalidation
    let cached_value = if config.cache_control.no_cache {
        Entry::Miss
    } else {
        cache
            .get(&key)
            .instrument(info_span!("cache_get"))
            .await
            .unwrap_or_else(|e| {
                log::warn!(ctx.ray_id(), "Error loading {key} from cache: {e}",);
                Entry::Miss
            })
    };

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
                    config.common_cache_tags.clone(),
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
            let origin_result = value_fut.await?;
            let origin_tags = origin_result.cache_tags(config.common_cache_tags.clone());

            if origin_result.should_purge_related() {
                let ray_id = ctx.ray_id().to_string();
                let cache = Arc::clone(cache);
                let purge_cache_tags = origin_tags.clone();

                log::info!(ray_id, "Purging global cache by tags: {:?}", purge_cache_tags);

                ctx.wait_until(
                    make_send_on_wasm(async move {
                        if let Err(err) = cache
                            .purge_by_tags(purge_cache_tags)
                            .instrument(info_span!("cache_tags_purge"))
                            .await
                        {
                            log::error!(ray_id, "Error global cache purge by tags: {err}");
                        }
                    })
                    .boxed(),
                )
                .await;
            }

            if origin_result.should_cache() && !config.cache_control.no_store {
                let ray_id = ctx.ray_id().to_string();
                let put_value = origin_result.clone();
                let cache = Arc::clone(cache);

                ctx.wait_until(
                    make_send_on_wasm(async move {
                        if let Err(err) = cache
                            .put(&key, EntryState::Fresh, put_value, origin_tags)
                            .instrument(info_span!("cache_put"))
                            .await
                        {
                            log::error!(ray_id, "Error cache PUT: {err}");
                        }
                    })
                    .boxed(),
                )
                .await;

                return Ok(CacheResponse::Miss(origin_result));
            }

            Ok(CacheResponse::Bypass(origin_result))
        }
    }
}

async fn update_stale<Value, Error, ValueFut>(
    cache: &Arc<impl Cache<Value = Value> + 'static>,
    ctx: &impl RequestContext,
    key: String,
    existing_value: Arc<Value>,
    value_fut: ValueFut,
    priority_tags: Vec<String>,
) where
    Value: Cacheable + 'static,
    Error: std::fmt::Display + Send,
    ValueFut: Future<Output = Result<Arc<Value>, Error>> + Send + 'static,
{
    let ray_id = ctx.ray_id().to_string();
    let existing_tags = existing_value.cache_tags(priority_tags.clone());

    // refresh the cache async and update the existing entry state
    let cache = Arc::clone(cache);
    ctx.wait_until(
        make_send_on_wasm(async move {
            let put_futures = cache
                .put(
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
                        .put(&key, EntryState::Fresh, fresh_value, fresh_cache_tags)
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
                        .put(&key, EntryState::Stale, existing_value, existing_tags)
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
    )
    .await;
}

#[cfg(test)]
mod tests {
    use super::key::{CacheAccess, CacheKey};
    use super::*;
    use crate::Error;
    use engine::parser::types::OperationType;
    use engine::registry::{CacheAccessScope, MetaField, MetaFieldType, MetaType, ObjectType, Registry};
    use tokio::sync::{Mutex, RwLock};

    use std::collections::hash_map::DefaultHasher;
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    use common_types::auth::ExecutionAuth;
    use engine::Request;
    use futures_util::future::BoxFuture;
    use runtime::cache;
    use runtime::cache::test_utils::FakeCache;

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
    struct FakeRequestContext {
        headers: Vec<(&'static str, &'static str)>,
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

        fn header(&self, name: &str) -> Option<String> {
            self.headers
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| (*v).to_string())
        }

        fn authorization_header(&self) -> Option<String> {
            unimplemented!()
        }

        fn headers(&self) -> HashMap<String, String> {
            unimplemented!()
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
            &config(None),
            &FakeRequestContext::default(),
            Request::new(QUERY),
            ExecutionAuth::ApiKey,
            |_, _| async { Ok::<_, Error>(res2) },
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
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                Ok(cache::Entry::Miss)
            }

            async fn put(
                &self,
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
            &CacheConfig {
                global_enabled: true,
                ..config(None)
            },
            &FakeRequestContext::default(),
            Request::new(QUERY),
            ExecutionAuth::ApiKey,
            |_, _| async { Ok::<_, Error>(res2) },
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
            CacheConfig {
                global_enabled: true,
                subdomain: TEST.to_string(),
                ..config(None)
            },
            FakeRequestContext::default(),
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
            CacheConfig {
                global_enabled: true,
                subdomain: TEST.to_string(),
                ..config(Some(engine::CacheControl {
                    access_scopes: Some([CacheAccessScope::ApiKey].into()),
                    ..Default::default()
                }))
            },
            FakeRequestContext::default(),
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
            CacheConfig {
                global_enabled: true,
                subdomain: TEST.to_string(),
                ..config(Some(engine::CacheControl {
                    access_scopes: Some(
                        [CacheAccessScope::Jwt {
                            claim: TEST.to_string(),
                        }]
                        .into(),
                    ),
                    ..Default::default()
                }))
            },
            FakeRequestContext::default(),
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
            CacheConfig {
                global_enabled: true,
                subdomain: TEST.to_string(),
                ..config(Some(engine::CacheControl {
                    access_scopes: Some(
                        [CacheAccessScope::Header {
                            header: TEST.to_string(),
                        }]
                        .into(),
                    ),
                    ..Default::default()
                }))
            },
            FakeRequestContext {
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
            CacheConfig {
                global_enabled: true,
                subdomain: TEST.to_string(),
                ..config(Some(engine::CacheControl {
                    access_scopes: Some([CacheAccessScope::Public].into()),
                    ..Default::default()
                }))
            },
            FakeRequestContext::default(),
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
            &CacheConfig {
                global_enabled: true,
                subdomain: TEST.to_string(),
                ..config(Some(engine::CacheControl {
                    access_scopes: Some(
                        [CacheAccessScope::Header {
                            header: TEST.to_string(),
                        }]
                        .into(),
                    ),
                    ..Default::default()
                }))
            },
            &FakeRequestContext::default(),
            Request::new(QUERY),
            ExecutionAuth::ApiKey,
            |_, _| async { Ok::<_, Error>(res2) },
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

    async fn expect_cache_key(config: CacheConfig<'_>, ctx: FakeRequestContext, auth: ExecutionAuth, expected: String) {
        #[derive(Default)]
        struct TestCache(tokio::sync::Mutex<Vec<String>>);

        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                self.0.lock().await.push(key.to_string());
                Ok(cache::Entry::Hit(Dummy::default()))
            }

            async fn put(
                &self,
                _key: &str,
                _state: cache::EntryState,
                _value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> cache::Result<()> {
                Ok(())
            }
        }

        let cache = Arc::new(TestCache::default());
        let response = execute_with_cache(&cache, &config, &ctx, Request::new(QUERY), auth, |_, _| async {
            Ok::<_, Error>(dummy(""))
        })
        .await
        .unwrap();

        assert_eq!(response, ExecutionResponse::Cached(dummy("")));
        assert_eq!(Arc::into_inner(cache).unwrap().0.into_inner(), vec![expected]);
    }

    fn config(cache_control: Option<engine::CacheControl>) -> CacheConfig<'static> {
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
        registry.enable_caching = true;
        CacheConfig {
            global_enabled: false,
            cache_control: Default::default(),
            common_cache_tags: vec![],
            subdomain: String::new(),
            partial_registry: Cow::Owned(registry.into()),
            host_name: String::new(),
        }
    }

    #[tokio::test]
    async fn should_successfully_get_miss() {
        static GET_CALLS: AtomicUsize = AtomicUsize::new(0);
        static PUT_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct TestCache;
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(cache::Entry::Miss)
            }

            async fn put(
                &self,
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
        let ctx = FakeRequestContext::default();
        let response = cached(
            &Arc::new(TestCache),
            &config(None),
            &ctx,
            "cache_key".to_string(),
            async { Ok::<_, Error>(dummy2) },
        )
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
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(cache::Entry::Hit(Dummy {
                    value: "cached".to_string(),
                    ..Default::default()
                }))
            }
        }

        let ctx = FakeRequestContext::default();
        let response = cached(
            &Arc::new(TestCache),
            &config(None),
            &ctx,
            "cache_key".to_string(),
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
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
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
                _key: &str,
                state: cache::EntryState,
                value: Arc<Self::Value>,
                _tags: Vec<String>,
            ) -> cache::Result<()> {
                self.put_calls.write().await.push((state, value));
                Ok(())
            }
        }

        let ctx = FakeRequestContext::default();
        let cache = Arc::new(TestCache::default());
        let response = cached(&cache, &config(None), &ctx, "cache_key".to_string(), async {
            Ok::<_, Error>(dummy("new"))
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
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
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

        let ctx = FakeRequestContext::default();
        let cache = Arc::new(TestCache::default());
        let response = cached(&cache, &config(None), &ctx, "cache_key".to_string(), async {
            Err(Error::BadRequest("failed_source".to_string()))
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
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
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

        let ctx = FakeRequestContext::default();
        let response = cached(
            &Arc::new(TestCache),
            &config(None),
            &ctx,
            "cache_key".to_string(),
            async { Err(Error::BadRequest("failed_source".to_string())) },
        )
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
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                GET_CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(cache::Entry::Miss)
            }
        }

        let ctx = FakeRequestContext::default();
        let dummy = Arc::new(Dummy {
            value: "new".to_string(),
            operation_type: OperationType::Mutation,
            ..Default::default()
        });
        let dummy2 = Arc::clone(&dummy);
        let response = cached(
            &Arc::new(TestCache),
            &config(None),
            &ctx,
            "cache_key".to_string(),
            async { Ok::<_, Error>(dummy2) },
        )
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
        #[async_trait::async_trait]
        impl FakeCache for TestCache {
            type Value = Dummy;

            async fn get(&self, _key: &str) -> cache::Result<cache::Entry<Self::Value>> {
                self.get_calls.fetch_add(1, Ordering::SeqCst);
                Ok(Entry::Miss)
            }

            async fn purge_by_tags(&self, tags: Vec<String>) -> cache::Result<()> {
                self.purge_calls.write().await.push(tags);
                Ok(())
            }
        }

        let cfg = CacheConfig {
            common_cache_tags: vec!["project".to_string()],
            ..config(None)
        };
        let ctx = FakeRequestContext::default();
        let dummy = Arc::new(Dummy {
            value: "new".to_string(),
            operation_type: OperationType::Mutation,
            tags: vec!["tag".into()],
            ..Default::default()
        });
        let dummy2 = Arc::clone(&dummy);
        let cache = Arc::new(TestCache::default());
        let response = cached(&cache, &cfg, &ctx, "cache_key".to_string(), async {
            Ok::<_, Error>(dummy2)
        })
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
