#![allow(unused_crate_dependencies)]

mod pg;

use common_types::auth::ExecutionAuth;
use engine::Request;
use gateway_core::{AdminAuthError, AuthError};
use runtime_noop::cache::NoopCache;
use std::{pin::Pin, sync::Arc};
use wasm_bindgen::prelude::*;

struct Executor {
    schema: engine::Schema,
}

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
        _ctx: Arc<Self::Context>,
        _auth: ExecutionAuth,
        request: Request,
    ) -> Result<engine::Response, Self::Error> {
        Ok(self.schema.execute(request).await)
    }

    async fn execute_stream(
        self: Arc<Self>,
        _ctx: Arc<Self::Context>,
        _auth: ExecutionAuth,
        _request: engine::Request,
        _streaming_format: gateway_core::StreamingFormat,
    ) -> Result<Self::Response, Self::Error> {
        Err(ResponseError("Streaming responses are not supported".to_string()))
    }
}

#[wasm_bindgen]
pub struct GrafbaseGateway {
    gateway: gateway_core::Gateway<Executor, NoopCache<engine::Response>>,
}

#[wasm_bindgen]
impl GrafbaseGateway {
    #[wasm_bindgen(constructor)]
    pub fn new(schema: &str) -> Result<GrafbaseGateway, JsValue> {
        console_error_panic_hook::set_once();
        tracing_wasm::set_as_global_default();

        let registry: engine::Registry =
            serde_json::from_str(schema).map_err(|err| JsValue::from(format!("Error reading config: {err}")))?;
        let schema = engine::Schema::build(registry)
            .data(engine::registry::resolvers::graphql::QueryBatcher::new())
            .data(pg::make_pg_transport_factory())
            .finish();
        let executor = Arc::new(Executor { schema });
        let cache = Arc::new(NoopCache::<engine::Response>::new());
        let cache_config = Default::default();
        let authorizer = Box::new(Authorizer);
        let gateway = gateway_core::Gateway::new(executor, cache, cache_config, authorizer);

        tracing::info!("new worked");

        Ok(GrafbaseGateway { gateway })
    }

    #[wasm_bindgen]
    pub async fn execute(&self, request: String) -> Result<String, JsValue> {
        tracing::info!("in execute");
        let ctx = Arc::new(Context {
            headers: http::HeaderMap::new(),
        });
        let request: engine::Request = serde_json::from_str(&request).map_err(|err| JsValue::from(err.to_string()))?;
        let response = self
            .gateway
            .execute(&ctx, request, None)
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
        Ok(response.0)
    }
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
        _ctx: &Arc<Self::Context>,
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

    fn with_additional_headers(self, _headers: http::HeaderMap) -> Self {
        self
    }

    fn error(code: http::StatusCode, message: &str) -> Self {
        Response(
            serde_json::to_string(&serde_json::json!({ "errors": [{ "message": format!("[{code}] {message}")}] }))
                .unwrap(),
        )
    }

    fn engine(response: Arc<engine::Response>) -> Result<Self, Self::Error> {
        Ok(Response(
            serde_json::to_string(&response.to_graphql_response()).unwrap(),
        ))
    }

    fn admin(response: async_graphql::Response) -> Result<Self, Self::Error> {
        Ok(Response(serde_json::to_string(&response).unwrap()))
    }
}
