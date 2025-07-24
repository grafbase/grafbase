use engine_error::{ErrorResponse, GraphqlError};
use event_queue::EventQueue;
use http::{request, response};
use runtime::extension::{EngineHooksExtension, GatewayHooksExtension, OnRequest, ReqwestParts};

use crate::{
    WasmContext,
    extension::{EngineWasmExtensions, GatewayWasmExtensions},
    wasmsafe,
};

impl GatewayHooksExtension<WasmContext> for GatewayWasmExtensions {
    async fn on_request(&self, parts: request::Parts) -> Result<OnRequest<WasmContext>, ErrorResponse> {
        let context = WasmContext::new(EventQueue::new(self.hooks_event_filter));
        let Some(pool) = self.hooks.as_ref() else {
            return Ok(OnRequest {
                context,
                parts,
                contract_key: None,
            });
        };

        let mut instance = pool.get().await.map_err(|err| {
            tracing::error!("Failed to get instance from pool: {err}");
            ErrorResponse::internal_extension_error()
        })?;

        wasmsafe!(instance.on_request(context, parts).await)
    }

    async fn on_response(&self, context: WasmContext, parts: response::Parts) -> Result<response::Parts, String> {
        let Some(pool) = self.hooks.as_ref() else {
            return Ok(parts);
        };
        let mut instance = pool.get().await.map_err(|e| e.to_string())?;

        wasmsafe!(instance.on_response(context, parts).await)
    }
}

impl EngineHooksExtension<WasmContext> for EngineWasmExtensions {
    async fn on_subgraph_request(
        &self,
        context: &WasmContext,
        parts: ReqwestParts,
    ) -> Result<ReqwestParts, GraphqlError> {
        let Some(pool) = self.gateway_extensions.hooks.as_ref() else {
            return Ok(parts);
        };
        let mut instance = pool.get().await.map_err(|e| {
            tracing::error!("Failed to get instance from pool: {e}");
            GraphqlError::internal_extension_error()
        })?;

        wasmsafe!(instance.on_subgraph_request(context, parts).await)
    }
}
