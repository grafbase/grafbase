use common_types::auth::ExecutionAuth;
use engine::Request;
use gateway_core::{AdminAuthError, AuthError};
use runtime_noop::cache::NoopCache;
use std::{pin::Pin, sync::Arc};
use wasm_bindgen::prelude::*;

struct Executor;
struct Context {
    headers: http::HeaderMap,
}

#[async_trait::async_trait]
impl gateway_core::RequestContext for Context {
    fn ray_id(&self) -> &str {
        "nope"
    }

    // Request execution will wait for those futures to end.
    // worker requires a 'static future, so there isn't any choice.
    async fn wait_until(&self, fut: Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>>) {
        fut.await
    }

    fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }
}

#[async_trait::async_trait]
impl gateway_core::Executor for Executor {
    type Error = ResponseError;
    type Context = Context;
    type Response = Response;

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
        Err(ResponseError("Streaming responses are not supported".to_string()))
    }
}

struct GrafbaseGateway {
    gateway: gateway_core::Gateway<Executor, ()>,
}

struct Authorizer;

#[async_trait::async_trait]
impl gateway_core::Authorizer for Authorizer {
    type Context = Context;

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

pub struct Response(String);

#[derive(Debug)]
pub struct ResponseError(String);

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ResponseError {}

impl From<gateway_core::Error> for ResponseError {
    fn from(err: gateway_core::Error) -> Self {
        ResponseError(err.to_string())
    }
}

impl gateway_core::Response for Response {
    type Error = ResponseError;

    fn with_additional_headers(self, headers: http::HeaderMap) -> Self {
        todo!()
    }

    fn error(code: http::StatusCode, message: &str) -> Self {
        todo!()
    }

    fn engine(response: Arc<engine::Response>) -> Result<Self, Self::Error> {
        todo!()
    }

    fn admin(response: async_graphql::Response) -> Result<Self, Self::Error> {
        todo!()
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
