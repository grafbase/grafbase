use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

use auth::Authorizer;
use engine::{Request, RequestHeaders};
use engine_v2::{Engine, EngineEnv, ExecutionMetadata, PreparedExecution, Schema};
use futures_util::Stream;
use gateway_core::RequestContext;
use headers::HeaderMapExt;
use runtime::cache::{CacheReadStatus, CachedExecutionResponse};

pub mod streaming;

pub struct Gateway {
    engine: Arc<Engine>,
    env: GatewayEnv,
    authorizer: Box<dyn Authorizer>,
}

pub struct GatewayEnv {
    pub kv: runtime::kv::KvStore,
    pub cache: runtime::cache::Cache,
}

pub struct Response {
    pub status: http::StatusCode,
    pub headers: http::HeaderMap,
    pub bytes: Vec<u8>,
    pub metadata: ExecutionMetadata,
    pub has_errors: bool,
}

impl Gateway {
    pub fn new(schema: Schema, engine_env: EngineEnv, env: GatewayEnv) -> Self {
        let authorizer = auth::build(schema.auth_config.as_ref(), &env.kv);
        let engine = Arc::new(Engine::new(schema, engine_env));
        Self {
            engine,
            env,
            authorizer,
        }
    }

    // The Engine is directly accessible
    pub async fn unchecked_engine_execute(&self, ctx: &impl RequestContext, request: Request) -> Response {
        let request_headers = headers(ctx);
        let response = self.engine.execute(request, request_headers).await;
        let has_errors = response.has_errors();
        match serde_json::to_vec(&response) {
            Ok(bytes) => Response {
                status: http::StatusCode::OK,
                headers: http::HeaderMap::new(),
                bytes,
                metadata: response.take_metadata(),
                has_errors,
            },
            Err(_) => Response {
                status: http::StatusCode::INTERNAL_SERVER_ERROR,
                headers: http::HeaderMap::new(),
                bytes: serde_json::to_vec(&serde_json::json!({
                    "errors": [{"message": "Server error"}],
                }))
                .expect("Obviously valid JSON"),
                metadata: ExecutionMetadata::default(),
                has_errors,
            },
        }
    }

    pub async fn execute(&self, ctx: &impl RequestContext, request: Request) -> Response {
        let request_headers = headers(ctx);
        let cached_response = if let Some(token) = self.authorizer.get_access_token(&request_headers).await {
            let prepared_execution = self.engine.execute(request, request_headers);
            match self.build_cache_key(&prepared_execution, &token) {
                Some(key) => {
                    self.env
                        .cache
                        .cached_execution(ctx, key, async move {
                            prepared_execution
                                .await
                                .into_cacheable(serde_json::to_vec)
                                .map(Arc::new)
                        })
                        .await
                }
                None => prepared_execution
                    .await
                    .into_cacheable(serde_json::to_vec)
                    .map(|response| CachedExecutionResponse::Origin {
                        response: Arc::new(response),
                        cache_read: CacheReadStatus::Bypass,
                    }),
            }
        } else {
            engine_v2::Response::error("Unauthorized")
                .into_cacheable(serde_json::to_vec)
                .map(|bytes| CachedExecutionResponse::Origin {
                    response: Arc::new(bytes),
                    cache_read: CacheReadStatus::Bypass,
                })
        };
        let mut response = cached_response
            .map(CachedExecutionResponse::into_response_and_headers)
            .map(|(response, headers)| {
                // If it wasn't cached, we're the only owner of it. No need to clone.
                let response = Arc::try_unwrap(response).unwrap_or_else(|response| response.as_ref().clone());
                Response {
                    status: http::StatusCode::OK,
                    bytes: response.bytes,
                    metadata: response.metadata,
                    has_errors: response.has_errors,
                    headers,
                }
            })
            .unwrap_or_else(|err| {
                log::error!(ctx.ray_id(), "Serialization error: {err}");
                Response {
                    status: http::StatusCode::INTERNAL_SERVER_ERROR,
                    bytes: serde_json::to_vec(&serde_json::json!({
                        "errors": [{"message": "Server error"}],
                    }))
                    .expect("Obviously valid JSON"),
                    metadata: ExecutionMetadata::default(),
                    has_errors: true,
                    headers: http::HeaderMap::new(),
                }
            });
        response
            .headers
            .typed_insert::<headers::ContentType>(headers::ContentType::json());

        response
    }

    pub fn execute_stream(
        &self,
        ctx: impl RequestContext,
        request: engine::Request,
    ) -> impl Stream<Item = engine_v2::Response> {
        self.engine.execute_stream(request, headers(&ctx))
    }

    fn build_cache_key(
        &self,
        prepared_execution: &PreparedExecution,
        token: &auth::AccessToken,
    ) -> Option<runtime::cache::Key> {
        let PreparedExecution::PreparedRequest(prepared) = prepared_execution else {
            return None;
        };
        // necessary later for cache scopes and if there is no cache config, there isn't any key.
        let _cache_config = prepared.computed_cache_config()?;
        let mut hasher = DefaultHasher::new();
        prepared.operation_hash(&mut hasher);
        token.hash(&mut hasher);
        let h = hasher.finish();

        Some(self.env.cache.build_key(&h.to_string()))
    }
}

fn headers(ctx: &impl RequestContext) -> RequestHeaders {
    RequestHeaders::from_iter(
        ctx.headers()
            .iter()
            .map(|(name, value)| (name.to_string(), String::from_utf8_lossy(value.as_bytes()).to_string())),
    )
}
