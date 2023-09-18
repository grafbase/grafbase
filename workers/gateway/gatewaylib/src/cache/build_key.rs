use std::{
    collections::{hash_map::DefaultHasher, BTreeSet},
    sync::Arc,
    time::Duration,
};

use common_types::auth::ExecutionAuth;
use engine::registry::CacheAccessScope;
use futures_util::TryFutureExt;
use gateway_adapter::{ExecutionEngine, ExecutionRequest};
use tracing_futures::Instrument;
use worker::Result;

use crate::{
    cache::{Cache, CacheAccess, CacheKey, CacheProvider, CacheReadStatus, CacheResponse, Cacheable},
    RequestContext,
};

pub enum ExecutionResponse<T> {
    Stale {
        response: Arc<T>,
        cache_revalidation: bool,
    },
    Cached(Arc<T>),
    Origin {
        response: Arc<T>,
        cache_read: Option<CacheReadStatus>,
    },
    Forbidden(Arc<T>),
}

impl<T: Cacheable> From<CacheResponse<Arc<T>>> for ExecutionResponse<T> {
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

pub struct ExecutionUpstream<E, CV, CP> {
    cache: Option<Cache<CV, CP>>,
    execution_engine: Arc<E>,
}

impl<ConfigType, E, CV, CP> ExecutionUpstream<E, CV, CP>
where
    E: ExecutionEngine<ConfigType = ConfigType, ExecutionResponse = CV> + 'static,
    CV: Cacheable + 'static,
    CP: CacheProvider<Value = CV>,
{
    pub fn new(cache: Option<Cache<CV, CP>>, execution_engine: E) -> Self {
        Self {
            cache,
            execution_engine: Arc::new(execution_engine),
        }
    }

    pub async fn health(
        &self,
        health_execution_request: gateway_adapter::ExecutionHealthRequest<ConfigType>,
    ) -> Result<gateway_adapter::ExecutionHealthResponse> {
        self.execution_engine
            .clone()
            .health(health_execution_request)
            .instrument(tracing::info_span!("execution_health"))
            .await
            .map_err(|err| worker::Error::RustError(err.to_string()))
    }

    pub async fn execute(
        &self,
        request_context: &RequestContext,
        execution_request: ExecutionRequest<ConfigType>,
    ) -> Result<ExecutionResponse<E::ExecutionResponse>> {
        match &self.cache {
            Some(cache) => {
                let cache_key = build_cache_key(&execution_request);

                let execution_engine = Arc::clone(&self.execution_engine);
                let execution_future = execution_engine
                    .execute(execution_request)
                    .map_err(|err| worker::Error::RustError(err.to_string()));

                let Ok(cache_key) = cache_key else {
                    log::debug!(
                        request_context.cloudflare_request_context.ray_id,
                        "error creating cache key, bypassing cache: {}",
                        cache_key.err().unwrap(),
                    );

                    return Ok(ExecutionResponse::Origin {
                        response: Arc::new(execution_future.await?),
                        cache_read: Some(CacheReadStatus::Bypass),
                    });
                };

                // cached execution
                let cache_response: CacheResponse<Arc<CV>> = cache
                    .cached(request_context, &cache_key, execution_future)
                    .instrument(tracing::info_span!("cached_execution"))
                    .await?;

                Ok(cache_response.into())
            }
            None => {
                let execution_response = self
                    .execution_engine
                    .clone()
                    .execute(execution_request)
                    .map_err(|err| worker::Error::RustError(err.to_string()))
                    .instrument(tracing::info_span!("origin_execution"))
                    .await?;

                Ok(ExecutionResponse::Origin {
                    response: Arc::new(execution_response),
                    cache_read: None,
                })
            }
        }
    }
}

fn build_cache_key<ConfigType>(execution_request: &ExecutionRequest<ConfigType>) -> Result<String> {
    // cache key access
    let request_cache_control = execution_request
        .parse()
        .map_err(|err| worker::Error::RustError(err.to_string()))?;

    let cache_access = request_cache_control
        .access_scopes
        .map(|scopes| {
            let cache_key_access = scopes.iter().fold(BTreeSet::new(), |mut current_scopes, scope| {
                match scope {
                    CacheAccessScope::Public | CacheAccessScope::ApiKey => {
                        current_scopes.insert(execution_request.auth.global_ops().to_string());
                    }
                    CacheAccessScope::Jwt { claim } => {
                        if let ExecutionAuth::Token(token) = &execution_request.auth {
                            if let Some(claim_value) = token.get_claim(claim) {
                                current_scopes.insert(claim_value);
                            }
                        }
                    }
                    CacheAccessScope::Header { header: name } => {
                        if let Some(header_value) = execution_request.execution_headers.get(name) {
                            current_scopes.insert(header_value.to_string());
                        }
                    }
                };

                current_scopes
            });

            CacheAccess::Scoped(cache_key_access)
        })
        .unwrap_or(CacheAccess::Default(&execution_request.auth));

    match &cache_access {
        CacheAccess::Scoped(scopes) if scopes.is_empty() => {
            return Err(worker::Error::RustError("Not a single cache scope matched".to_string()));
        }
        _ => {}
    }

    // cache key
    // note: I opted for using `DefaultHasher` as its using SipHash-2-4.
    // this hashing algorithm is *not* collision resistant but it provides a good mix of security and speed
    // using cryptographic hashes provide a more secure alternative as they are collision resistant BUT are slower
    // additionally, each combination of <project>-<branch> gets their own cache in order to reduce the number keys directed to a particular cache
    // note: I'm also using DefaultHasher and not SipHash24 because SipHash direct usage is deprecated.
    // But beware that the default hash implementation can change across rust releases so pay attention to that when bumping
    let cache_key = CacheKey::<DefaultHasher>::new(
        cache_access,
        &execution_request.request,
        &execution_request.config.common.subdomain,
    );

    Ok(format!(
        "https://{}/{}",
        &execution_request.config.common.subdomain,
        cache_key.to_hash_string()
    ))
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet, HashMap},
        sync::Arc,
        time::Duration,
    };

    use async_trait::async_trait;
    use common_types::auth::ExecutionAuth;
    use engine::{
        registry::{CacheAccessScope, CacheConfig, MetaField, MetaFieldType, MetaType, ObjectType, Registry},
        CacheControl, Request,
    };
    use futures_util::future::BoxFuture;
    use gateway_adapter::{
        CommonCustomerDeploymentConfig, CustomerDeploymentConfig, ExecutionEngine, ExecutionHealthRequest,
        ExecutionHealthResponse, ExecutionRequest, ExecutionResult, StreamingFormat,
    };
    use gateway_adapter_platform::PlatformSpecificConfig;
    use rusoto_core::Region;
    use serde_json::Value;

    use crate::{
        cache::{
            Cache, CacheAccess, CacheEntryState, CacheKey, CacheProvider, CacheProviderResponse, CacheReadStatus,
            CacheResult, Cacheable, EdgeCache, NoopGlobalCache,
        },
        platform::{
            context::RequestContext,
            upstream::{ExecutionResponse, ExecutionUpstream},
        },
    };

    const TEST: &str = "Test";

    #[derive(serde::Serialize, serde::Deserialize, Default, Eq, PartialEq, Debug)]
    struct TestCacheableType;
    impl Cacheable for TestCacheableType {
        fn max_age_seconds(&self) -> usize {
            1
        }

        fn stale_seconds(&self) -> usize {
            2
        }

        fn ttl_seconds(&self) -> usize {
            3
        }

        fn cache_tags(&self, _priority_tags: Vec<String>) -> Vec<String> {
            Vec::new()
        }

        fn should_purge_related(&self) -> bool {
            true
        }

        fn should_cache(&self) -> bool {
            true
        }
    }

    struct TestExecutionEngine;
    #[async_trait(?Send)]
    impl ExecutionEngine for TestExecutionEngine {
        type ConfigType = PlatformSpecificConfig;
        type ExecutionResponse = TestCacheableType;

        async fn execute(
            self: Arc<Self>,
            _execution_request: ExecutionRequest<PlatformSpecificConfig>,
        ) -> ExecutionResult<TestCacheableType> {
            Ok(Default::default())
        }

        async fn execute_stream(
            self: Arc<Self>,
            _execution_request: ExecutionRequest<Self::ConfigType>,
            _streaming_format: StreamingFormat,
        ) -> ExecutionResult<(worker::Response, Option<BoxFuture<'static, ()>>)> {
            unimplemented!()
        }

        async fn health(
            self: Arc<Self>,
            _execution_health_request: ExecutionHealthRequest<gateway_adapter_platform::PlatformSpecificConfig>,
        ) -> ExecutionResult<ExecutionHealthResponse> {
            Ok(ExecutionHealthResponse {
                deployment_id: String::new(),
                ready: true,
                udf_results: vec![],
            })
        }
    }

    impl Default for RequestContext {
        fn default() -> Self {
            Self {
                closest_aws_region: Region::Custom {
                    name: String::new(),
                    endpoint: String::new(),
                },
                #[cfg(not(feature = "local"))]
                api_key_auth: Default::default(),
                cloudflare_request_context: Default::default(),
                config: Arc::new(Default::default()),
                wait_until_promises: Arc::new(RefCell::new(vec![])),
            }
        }
    }

    macro_rules! assert_cache_hit_execution {
        ($gql_query: ident, $cache_config: ident, $request_auth: ident, $request_headers: ident, $expected_cache_key: ident) => {{
            static mut CACHE_KEY_ARG_CAPTURE: String = String::new();

            struct TestCache;
            #[async_trait(?Send)]
            impl CacheProvider for TestCache {
                type Value = TestCacheableType;

                async fn get(_cache_name: &str, key: &str) -> CacheResult<CacheProviderResponse<Self::Value>> {
                    unsafe {
                        CACHE_KEY_ARG_CAPTURE.push_str(key);
                    }
                    Ok(CacheProviderResponse::Hit(TestCacheableType))
                }

                async fn put(
                    _cache_name: &str,
                    _ray_id: &str,
                    _key: &str,
                    _status: CacheEntryState,
                    _value: Arc<Self::Value>,
                    _tags: Vec<String>,
                ) -> CacheResult<()> {
                    Ok(())
                }
            }

            let execution_response =
                cached_execution::<TestCache>(&$gql_query, TEST, $cache_config, $request_auth, $request_headers)
                    .await?;

            // assert
            if let ExecutionResponse::Cached(response) = execution_response {
                assert_eq!(response, Default::default());
                unsafe {
                    assert_eq!(CACHE_KEY_ARG_CAPTURE, $expected_cache_key);
                }
                Ok(())
            } else {
                Err(worker::Error::RustError("should be a Cached response".to_string()))
            }
        }};
    }

    #[tokio::test]
    async fn should_execute_without_cache() -> Result<(), worker::Error> {
        // prepare
        let execution_upstream = ExecutionUpstream::<
            TestExecutionEngine,
            TestCacheableType,
            EdgeCache<TestCacheableType>,
        >::new(None, TestExecutionEngine);

        let request_context: RequestContext = Default::default();
        let execution_request = ExecutionRequest {
            request: Request::new("query test { test }"),
            config: Default::default(),
            auth: ExecutionAuth::ApiKey,
            closest_aws_region: Default::default(),
            execution_headers: Default::default(),
        };

        // act
        let execution_response = execution_upstream.execute(&request_context, execution_request).await?;

        // assert
        if let ExecutionResponse::Origin { response, cache_read } = execution_response {
            assert_eq!(response, Default::default());
            assert_eq!(cache_read, None);
            Ok(())
        } else {
            Err(worker::Error::RustError("should be an Origin response".to_string()))
        }
    }

    #[tokio::test]
    async fn should_execute_with_cache_miss() -> Result<(), worker::Error> {
        // prepare
        struct TestCache;
        #[async_trait(?Send)]
        impl CacheProvider for TestCache {
            type Value = TestCacheableType;

            async fn get(_cache_name: &str, _key: &str) -> CacheResult<CacheProviderResponse<Self::Value>> {
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
                Ok(())
            }
        }

        // act
        let execution_response =
            cached_execution::<TestCache>("query { test { id } }", TEST, None, None, Default::default()).await?;

        // assert
        if let ExecutionResponse::Origin { response, cache_read } = execution_response {
            assert_eq!(response, Default::default());
            assert_eq!(
                cache_read.unwrap(),
                CacheReadStatus::Miss {
                    max_age: Duration::from_secs(1),
                }
            );
            Ok(())
        } else {
            Err(worker::Error::RustError("should be an Origin response".to_string()))
        }
    }

    #[tokio::test]
    async fn should_build_cache_key_for_auth() -> Result<(), worker::Error> {
        let (gql_query, _) = build_test_registry(BTreeSet::new());

        // expected cache key
        let engine_request = Request::new(gql_query.clone());
        let expected_cache_key =
            CacheKey::<DefaultHasher>::new(CacheAccess::Default(&ExecutionAuth::ApiKey), &engine_request, TEST);
        let expected_cache_key = format!("https://{}/{}", TEST, expected_cache_key.to_hash_string());

        let cache_config = None;
        let request_auth = None;
        let request_headers = HashMap::new();

        assert_cache_hit_execution!(
            gql_query,
            cache_config,
            request_auth,
            request_headers,
            expected_cache_key
        )
    }

    #[tokio::test]
    async fn should_build_cache_key_for_scoped_access_api_key() -> Result<(), worker::Error> {
        let (gql_query, registry) = build_test_registry(BTreeSet::from([CacheAccessScope::ApiKey]));

        // expected cache key
        let engine_request = Request::new(gql_query.clone());
        let expected_cache_key = CacheKey::<DefaultHasher>::new(
            CacheAccess::Scoped(BTreeSet::from([ExecutionAuth::ApiKey.global_ops().to_string()])),
            &engine_request,
            TEST,
        );
        let expected_cache_key = format!("https://{}/{}", TEST, expected_cache_key.to_hash_string());

        let cache_config = Some(registry.into());
        let request_auth = Some(ExecutionAuth::ApiKey);
        let request_headers = HashMap::new();

        assert_cache_hit_execution!(
            gql_query,
            cache_config,
            request_auth,
            request_headers,
            expected_cache_key
        )
    }

    #[tokio::test]
    async fn should_build_cache_key_for_scoped_access_jwt() -> Result<(), worker::Error> {
        // prepare
        let (gql_query, registry) = build_test_registry(BTreeSet::from([CacheAccessScope::Jwt {
            claim: TEST.to_string(),
        }]));

        // auth jwt
        let claim_value = Value::String(TEST.to_string());
        let request_auth = Some(ExecutionAuth::new_from_token(
            Default::default(),
            Default::default(),
            Default::default(),
            BTreeMap::from_iter([(TEST.to_string(), claim_value.clone())]),
        ));

        // expected cache key
        let engine_request = Request::new(gql_query.clone());
        let expected_cache_key = CacheKey::<DefaultHasher>::new(
            CacheAccess::Scoped(BTreeSet::from([claim_value.to_string()])),
            &engine_request,
            TEST,
        );
        let expected_cache_key = format!("https://{}/{}", TEST, expected_cache_key.to_hash_string());

        let cache_config = Some(registry.into());
        let request_headers = HashMap::new();

        assert_cache_hit_execution!(
            gql_query,
            cache_config,
            request_auth,
            request_headers,
            expected_cache_key
        )
    }

    #[tokio::test]
    async fn should_build_cache_key_for_scoped_access_header() -> Result<(), worker::Error> {
        // prepare
        let (gql_query, registry) = build_test_registry(BTreeSet::from([CacheAccessScope::Header {
            header: TEST.to_string(),
        }]));

        // expected cache key
        let engine_request = Request::new(gql_query.clone());
        let expected_cache_key = CacheKey::<DefaultHasher>::new(
            CacheAccess::Scoped(BTreeSet::from([TEST.to_string()])),
            &engine_request,
            TEST,
        );
        let expected_cache_key = format!("https://{}/{}", TEST, expected_cache_key.to_hash_string());

        let request_auth = None;
        let cache_config = Some(registry.into());
        let request_headers = HashMap::from([(TEST.to_string(), TEST.to_string())]);

        assert_cache_hit_execution!(
            gql_query,
            cache_config,
            request_auth,
            request_headers,
            expected_cache_key
        )
    }

    #[tokio::test]
    async fn should_build_cache_key_for_scoped_access_public() -> Result<(), worker::Error> {
        // prepare
        let (gql_query, registry) = build_test_registry(BTreeSet::from([CacheAccessScope::Public]));

        // expected cache key
        let engine_request = Request::new(gql_query.clone());
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

        let request_auth = Some(ExecutionAuth::Public {
            global_ops: Default::default(),
        });
        let cache_config = Some(registry.into());
        let request_headers = HashMap::new();

        assert_cache_hit_execution!(
            gql_query,
            cache_config,
            request_auth,
            request_headers,
            expected_cache_key
        )
    }

    #[tokio::test]
    async fn should_bypass_cache_on_missing_data_for_access_scopes() -> Result<(), worker::Error> {
        // prepare
        struct TestCache;
        impl CacheProvider for TestCache {
            type Value = TestCacheableType;
        }

        let cache = Cache::<TestCacheableType, TestCache>::new(TEST.to_string(), Box::new(NoopGlobalCache));
        let execution_upstream = ExecutionUpstream::new(Some(cache), TestExecutionEngine);

        let (gql_query, registry) = build_test_registry(BTreeSet::from([CacheAccessScope::Header {
            header: TEST.to_string(),
        }]));

        let request_context = Default::default();
        let execution_request = ExecutionRequest {
            request: Request::new(gql_query),
            config: CustomerDeploymentConfig {
                common: CommonCustomerDeploymentConfig {
                    subdomain: TEST.to_string(),
                    cache_config: registry.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            auth: ExecutionAuth::ApiKey,
            closest_aws_region: Default::default(),
            execution_headers: HashMap::new(),
        };

        // act
        let execution_response = execution_upstream.execute(&request_context, execution_request).await?;

        // assert
        if let ExecutionResponse::Origin { response, cache_read } = execution_response {
            assert_eq!(response, Default::default());
            assert_eq!(cache_read, Some(CacheReadStatus::Bypass));
            Ok(())
        } else {
            Err(worker::Error::RustError("should be an Origin response".to_string()))
        }
    }

    async fn cached_execution<T: CacheProvider<Value = TestCacheableType>>(
        gql_query: &str,
        subdomain: &str,
        cache_config: Option<CacheConfig>,
        auth: Option<ExecutionAuth>,
        headers: HashMap<String, String>,
    ) -> worker::Result<ExecutionResponse<TestCacheableType>> {
        let cache = Cache::<TestCacheableType, T>::new(TEST.to_string(), Box::new(NoopGlobalCache));
        let execution_upstream = ExecutionUpstream::new(Some(cache), TestExecutionEngine);

        let request_context: RequestContext = Default::default();

        let execution_request = ExecutionRequest {
            request: Request::new(gql_query),
            config: CustomerDeploymentConfig {
                common: CommonCustomerDeploymentConfig {
                    subdomain: subdomain.to_string(),
                    cache_config: cache_config.unwrap_or(Registry::new().into()),
                    ..Default::default()
                },
                ..Default::default()
            },
            auth: auth.unwrap_or(ExecutionAuth::ApiKey),
            closest_aws_region: Default::default(),
            execution_headers: headers,
        };

        execution_upstream.execute(&request_context, execution_request).await
    }

    fn build_test_registry(access_scopes: BTreeSet<CacheAccessScope>) -> (String, Registry) {
        let gql_query = "query { Test { id } }";

        let mut registry = Registry::new();
        registry.create_type(
            |_| {
                MetaType::Object(
                    ObjectType::new(TEST.to_string(), [MetaField::new("id", "String!")]).with_cache_control(
                        CacheControl {
                            access_scopes: Some(access_scopes),
                            ..Default::default()
                        },
                    ),
                )
            },
            TEST,
            TEST,
        );

        registry.query_root_mut().fields_mut().unwrap().insert(
            TEST.to_string(),
            MetaField::new(TEST.to_string(), MetaFieldType::from(TEST)),
        );

        (gql_query.to_string(), registry)
    }
}
