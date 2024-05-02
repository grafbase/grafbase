use std::collections::{hash_map::DefaultHasher, BTreeSet};
use tracing::instrument;

use common_types::auth::ExecutionAuth;
use engine::{
    registry::{CacheAccessScope, CacheControlError},
    CacheControl, ServerError,
};
use runtime::cache::Key;

use super::{
    key::{CacheAccess, CacheKey},
    CacheConfig,
};
use crate::RequestContext;

#[derive(thiserror::Error, Eq, PartialEq, Debug)]
pub enum BuildKeyError {
    #[error("Not a single cache scope matched")]
    MissingScope,
    #[error("Could not determine cache control: {0}")]
    CouldNotDetermineCacheControl(String),
}

#[instrument(skip_all)]
pub fn build_cache_key(
    config: &CacheConfig,
    ctx: &impl RequestContext,
    request: &engine::Request,
    auth: &ExecutionAuth,
) -> Result<Key, BuildKeyError> {
    let request_cache_control = get_cache_control(&config.partial_registry, request)
        .map_err(|err| BuildKeyError::CouldNotDetermineCacheControl(err.to_string()))?;

    let cache_access = request_cache_control
        .access_scopes
        .map(|scopes| {
            let cache_key_access = scopes.iter().fold(BTreeSet::new(), |mut current_scopes, scope| {
                match scope {
                    CacheAccessScope::Public | CacheAccessScope::ApiKey => {
                        current_scopes.insert(auth.global_ops().to_string());
                    }
                    CacheAccessScope::Jwt { claim } => {
                        if let ExecutionAuth::Token(token) = &auth {
                            if let Some(claim_value) = token.get_claim(claim) {
                                current_scopes.insert(claim_value);
                            }
                        }
                    }
                    CacheAccessScope::Header { header: name } => {
                        if let Some(header_value) = ctx.headers().get(name).and_then(|header| header.to_str().ok()) {
                            current_scopes.insert(header_value.to_string());
                        }
                    }
                };

                current_scopes
            });

            CacheAccess::Scoped(cache_key_access)
        })
        .unwrap_or(CacheAccess::Default(auth));

    match &cache_access {
        CacheAccess::Scoped(scopes) if scopes.is_empty() => return Err(BuildKeyError::MissingScope),
        _ => {}
    }

    // cache key
    // note: I opted for using `DefaultHasher` as its using SipHash-1-3.
    // this hashing algorithm is *not* collision resistant but it provides a good mix of security and speed
    // using cryptographic hashes provide a more secure alternative as they are collision resistant BUT are slower
    // additionally, each combination of <project>-<branch> gets their own cache in order to reduce the number keys directed to a particular cache
    // note: I'm also using DefaultHasher and not SipHash13 because SipHash direct usage is deprecated.
    // But beware that the default hash implementation can change across rust releases so pay attention to that when bumping
    let subdomain = &config.subdomain;
    let cache_key = CacheKey::<DefaultHasher>::new(cache_access, request, subdomain);

    Ok(Key::unchecked_new(format!(
        // v1 was stored in JSON, v2 in msgpack
        "https://{}/v2/{}",
        subdomain,
        cache_key.to_hash_string()
    )))
}

fn get_cache_control(
    registry: &registry_for_cache::PartialCacheRegistry,
    request: &engine::Request,
) -> Result<CacheControl, CacheControlError> {
    let document = engine_parser::parse_query(request.query()).map_err(CacheControlError::Parse)?;

    engine_validation::check_fast_rules(registry, &document, Some(&request.variables))
        .map(|res| res.cache_control)
        .map_err(|errors| CacheControlError::Validate(errors.into_iter().map(ServerError::from).collect()))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
        sync::Arc,
    };

    use futures_util::future::BoxFuture;
    use tokio::sync::Mutex;

    use crate::cache::build_cache_key;
    use common_types::auth::ExecutionAuth;
    use engine::{
        registry::{CacheAccessScope, MetaField, MetaFieldType, MetaType, ObjectType, Registry},
        Request,
    };
    use runtime::{cache::Key, context::RequestContext};

    use crate::cache::build_key::BuildKeyError;
    use crate::cache::key::{CacheAccess, CacheKey};
    use crate::CacheConfig;

    const TEST: &str = "Test";
    const QUERY: &str = "query { Test { id } }";

    #[derive(Default)]
    struct FakeRequestContext {
        headers: http::HeaderMap,
        futures: Mutex<Vec<BoxFuture<'static, ()>>>,
    }

    impl FakeRequestContext {
        fn with_header(mut self, name: &'static str, value: &'static str) -> Self {
            self.headers.insert(name, http::HeaderValue::from_static(value));
            self
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
    async fn should_build_cache_key_for_auth() {
        // prepare
        let engine_request = Request::new(QUERY);
        let expected_cache_key =
            CacheKey::<DefaultHasher>::new(CacheAccess::Default(&ExecutionAuth::ApiKey), &engine_request, TEST);
        let expected_cache_key =
            Key::unchecked_new(format!("https://{}/v2/{}", TEST, expected_cache_key.to_hash_string()));
        let cache_config = CacheConfig {
            global_enabled: true,
            subdomain: TEST.to_string(),
            ..config(None)
        };
        let ctx = FakeRequestContext::default();
        let auth = ExecutionAuth::ApiKey;

        // act
        let response = build_cache_key(&cache_config, &ctx, &engine_request, &auth).unwrap();

        // assert
        assert_eq!(expected_cache_key, response);
    }

    #[tokio::test]
    async fn should_build_cache_key_for_scoped_access_api_key() {
        // prepare
        let engine_request = Request::new(QUERY);
        let expected_cache_key = CacheKey::<DefaultHasher>::new(
            CacheAccess::Scoped(BTreeSet::from([ExecutionAuth::ApiKey.global_ops().to_string()])),
            &engine_request,
            TEST,
        );
        let expected_cache_key =
            Key::unchecked_new(format!("https://{}/v2/{}", TEST, expected_cache_key.to_hash_string()));
        let cache_config = CacheConfig {
            global_enabled: true,
            subdomain: TEST.to_string(),
            ..config(Some(engine::CacheControl {
                access_scopes: Some([CacheAccessScope::ApiKey].into()),
                ..Default::default()
            }))
        };
        let ctx = FakeRequestContext::default();

        // act
        let response = build_cache_key(&cache_config, &ctx, &engine_request, &ExecutionAuth::ApiKey).unwrap();

        // assert
        assert_eq!(expected_cache_key, response);
    }

    #[tokio::test]
    async fn should_build_cache_key_for_scoped_access_jwt() {
        // prepare
        let claim_value = serde_json::Value::String(TEST.to_string());
        let auth = ExecutionAuth::new_from_token(
            Default::default(),
            Default::default(),
            Default::default(),
            BTreeMap::from_iter([(TEST.to_string(), claim_value.clone())]),
        );
        let engine_request = Request::new(QUERY);
        let expected_cache_key = CacheKey::<DefaultHasher>::new(
            CacheAccess::Scoped(BTreeSet::from([claim_value.to_string()])),
            &engine_request,
            TEST,
        );
        let expected_cache_key =
            Key::unchecked_new(format!("https://{}/v2/{}", TEST, expected_cache_key.to_hash_string()));
        let cache_config = CacheConfig {
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
        };
        let ctx = FakeRequestContext::default();

        // act
        let response = build_cache_key(&cache_config, &ctx, &engine_request, &auth).unwrap();

        // assert
        assert_eq!(expected_cache_key, response);
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
        let expected_cache_key =
            Key::unchecked_new(format!("https://{}/v2/{}", TEST, expected_cache_key.to_hash_string()));
        let cache_config = CacheConfig {
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
        };
        let ctx = FakeRequestContext::default().with_header(TEST, TEST);

        // act
        let response = build_cache_key(&cache_config, &ctx, &engine_request, &ExecutionAuth::ApiKey).unwrap();

        // assert
        assert_eq!(expected_cache_key, response);
    }

    #[tokio::test]
    async fn should_build_cache_key_for_scoped_access_public() {
        // prepare
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
        let expected_cache_key =
            Key::unchecked_new(format!("https://{}/v2/{}", TEST, expected_cache_key.to_hash_string()));
        let cache_config = CacheConfig {
            global_enabled: true,
            subdomain: TEST.to_string(),
            ..config(Some(engine::CacheControl {
                access_scopes: Some([CacheAccessScope::Public].into()),
                ..Default::default()
            }))
        };
        let ctx = FakeRequestContext::default();
        let auth = ExecutionAuth::Public {
            global_ops: Default::default(),
        };

        // act
        let response = build_cache_key(&cache_config, &ctx, &engine_request, &auth).unwrap();

        // assert
        assert_eq!(expected_cache_key, response);
    }

    #[tokio::test]
    async fn should_bypass_cache_on_missing_data_for_access_scopes() {
        // prepare
        let cache_config = CacheConfig {
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
        };
        let ctx = FakeRequestContext::default();
        let request = Request::new(QUERY);
        let auth = ExecutionAuth::ApiKey;

        // act
        let response = build_cache_key(&cache_config, &ctx, &request, &auth);

        // assert
        assert!(response.is_err());
        assert_eq!(response.err().unwrap(), BuildKeyError::MissingScope)
    }

    fn config(cache_control: Option<engine::CacheControl>) -> CacheConfig {
        let mut registry = Registry::new();
        registry.add_builtins_to_registry();

        registry.create_type(
            |_| {
                MetaType::Object({
                    let obj = ObjectType::new(TEST.to_string(), [MetaField::new("id", "String!")]);
                    obj.with_cache_control(cache_control.map(Box::new))
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

        registry.remove_unused_types();
        let registry = registry.prune_for_caching_registry();
        let partial_registry = Arc::new(registry_upgrade::convert_v1_to_partial_cache_registry(registry));

        CacheConfig {
            global_enabled: false,
            common_cache_tags: vec![],
            subdomain: String::new(),
            partial_registry,
            host_name: String::new(),
        }
    }
}
