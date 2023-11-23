use common_types::auth::ExecutionAuth;
use engine::Request;
use gateway_core::{AdminAuthError, AuthError};
use runtime_noop::cache::NoopCache;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

struct Executor;

#[async_trait::async_trait]
impl gateway_core::Executor for Executor {
    type Error = String;
    type Context = ();
    type Response = engine::Response;

    async fn execute(
        self: Arc<Self>,
        ctx: Arc<Self::Context>,
        auth: ExecutionAuth,
        request: Request,
    ) -> Result<engine::Response, Self::Error> {
        todo!()
    }

    async fn execute_stream(
        self: Arc<Self>,
        ctx: Arc<Self::Context>,
        auth: ExecutionAuth,
        request: engine::Request,
        streaming_format: gateway_core::StreamingFormat,
    ) -> Result<Self::Response, Self::Error> {
        Err("Streaming responses are not supported".to_string())
    }
}

struct GrafbaseGateway {
    gateway: gateway_core::Gateway<Executor, ()>,
}

struct Authorizer;

#[async_trait::async_trait]
impl gateway_core::Authorizer for Authorizer {
    type Context = ();

    async fn authorize_admin_request(
        &self,
        _ctx: &Arc<Self::Context>,
        _request: &async_graphql::Request,
    ) -> Result<(), AdminAuthError> {
        Ok(())
    }

    async fn authorize_request(
        &self,
        ctx: &Arc<Self::Context>,
        _request: &engine::Request,
    ) -> Result<ExecutionAuth, AuthError> {
        Ok(ExecutionAuth::new_from_api_keys())
    }
}

#[wasm_bindgen]
pub fn make_config(schema: &str) -> Vec<u8> {
    let executor = Arc::new(Executor);
    let cache = Arc::new(NoopCache::new());
    let cache_config = Default::default();
    let authorizer = Box::new(Authorizer);
    let gateway = gateway_core::Gateway::new(executor, cache, cache_config, authorizer);

    b"meowmeowmeow".to_vec()
}
