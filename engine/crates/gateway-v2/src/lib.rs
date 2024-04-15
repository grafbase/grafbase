use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

use auth::AuthService;
use engine::{ErrorCode, Request, RequestHeaders};
use engine_v2::{Engine, EngineEnv, ExecutionMetadata, PreparedExecution, Schema};
use futures_util::Stream;
use gateway_core::RequestContext;
use headers::HeaderMapExt;
use runtime::{
    auth::AccessToken,
    cache::{CacheReadStatus, CachedExecutionResponse},
};

#[cfg(feature = "axum")]
pub mod local_server;

pub mod streaming;
pub mod websockets;

pub struct Gateway {
    engine: Arc<Engine>,
    env: GatewayEnv,
    auth: AuthService,
}

impl std::fmt::Debug for Gateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gateway").finish_non_exhaustive()
    }
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
        let auth = AuthService::new_v2(schema.settings.auth_config.clone().unwrap_or_default(), env.kv.clone());
        let engine = Arc::new(Engine::new(schema, engine_env));
        Self { engine, env, auth }
    }

    // The Engine is directly accessible
    pub async fn unchecked_engine_execute(&self, ctx: &impl RequestContext, request: Request) -> Response {
        let headers = build_request_headers(ctx.headers());
        let response = self
            .engine
            .execute(request, AccessToken::Anonymous, headers)
            .await
            .await;
        let has_errors = response.has_errors();
        match serde_json::to_vec(&response) {
            Ok(bytes) => Response {
                status: http::StatusCode::OK,
                headers: http::HeaderMap::new(),
                bytes,
                metadata: response.take_metadata(),
                has_errors,
            },
            Err(_) => Response::internal_server_error(),
        }
    }

    pub async fn authorize(self: &Arc<Self>, headers: &http::HeaderMap) -> Option<Session> {
        let token = self.auth.get_access_token(headers).await?;

        Some(Session {
            gateway: Arc::clone(self),
            token,
            headers: build_request_headers(headers),
        })
    }

    fn build_cache_key(
        &self,
        prepared_execution: &PreparedExecution,
        token: &AccessToken,
    ) -> Option<runtime::cache::Key> {
        let PreparedExecution::PreparedRequest(prepared) = prepared_execution else {
            return None;
        };
        // necessary later for cache scopes and if there is no cache config, there isn't any key.
        let cache_control = prepared.cache_control()?;
        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        let h = hasher.finish();

        Some(self.env.cache.build_key(&format!("{}_{h}", cache_control.key.0)))
    }
}

/// An authenticated gateway session
#[derive(Clone)]
pub struct Session {
    gateway: Arc<Gateway>,
    token: AccessToken,
    headers: RequestHeaders,
}

impl Session {
    pub async fn execute(self, ctx: &impl RequestContext, request: Request) -> Response {
        let prepared_execution = self
            .gateway
            .engine
            .execute(request, self.token.clone(), self.headers)
            .await;
        let cached_response = match self.gateway.build_cache_key(&prepared_execution, &self.token) {
            Some(key) => {
                self.gateway
                    .env
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
            .unwrap_or_else(|_err| {
                #[cfg(feature = "tracing")]
                tracing::error!("Serialization error: {_err}");
                Response::internal_server_error()
            });
        response
            .headers
            .typed_insert::<headers::ContentType>(headers::ContentType::json());

        response
    }

    pub fn execute_stream(self, request: engine::Request) -> impl Stream<Item = engine_v2::Response> {
        self.gateway.engine.execute_stream(request, self.token, self.headers)
    }
}

impl Response {
    pub fn unauthorized() -> Self {
        let response = engine_v2::Response::error("Unauthorized", []);
        Response {
            status: http::StatusCode::UNAUTHORIZED,
            headers: Default::default(),
            bytes: serde_json::to_vec(&response).expect("this serialization should be fine"),
            metadata: response.take_metadata(),
            has_errors: true,
        }
    }

    pub fn forbidden() -> Self {
        let response = engine_v2::Response::error("Forbidden", []);
        Response {
            status: http::StatusCode::FORBIDDEN,
            headers: Default::default(),
            bytes: serde_json::to_vec(&response).expect("this serialization should be fine"),
            metadata: response.take_metadata(),
            has_errors: true,
        }
    }

    fn internal_server_error() -> Self {
        let response = engine_v2::Response::error(
            "Internal Server Error",
            [(
                "code".to_string(),
                serde_json::Value::String(ErrorCode::InternalServerError.to_string()),
            )],
        );
        Response {
            status: http::StatusCode::INTERNAL_SERVER_ERROR,
            headers: Default::default(),
            bytes: serde_json::to_vec(&response).expect("this serialization should be fine"),
            metadata: response.take_metadata(),
            has_errors: true,
        }
    }
}

fn build_request_headers(headers: &http::HeaderMap) -> RequestHeaders {
    RequestHeaders::from_iter(
        headers
            .iter()
            .map(|(name, value)| (name.to_string(), String::from_utf8_lossy(value.as_bytes()).to_string())),
    )
}
