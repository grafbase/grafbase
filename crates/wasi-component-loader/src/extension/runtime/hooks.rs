use std::sync::Arc;

use engine::EngineOperationContext;
use engine_error::{ErrorResponse, GraphqlError};
use engine_schema::{GraphqlSubgraph, VirtualSubgraph};
use event_queue::EventQueue;
use http::{request, response};
use runtime::extension::{EngineHooksExtension, GatewayHooksExtension, OnRequest, ReqwestParts};

use crate::{
    extension::{EngineWasmExtensions, GatewayWasmExtensions},
    wasmsafe,
};

impl GatewayHooksExtension for GatewayWasmExtensions {
    async fn on_request(&self, parts: request::Parts) -> Result<OnRequest, ErrorResponse> {
        let event_queue = EventQueue::new(self.hooks_event_filter);
        let Some(pool) = self.hooks.as_ref() else {
            return Ok(OnRequest {
                parts,
                contract_key: None,
                event_queue: Arc::new(event_queue),
                hooks_context: Default::default(),
            });
        };

        let mut instance = pool.get().await.map_err(|err| {
            tracing::error!("Failed to get instance from pool: {err}");
            ErrorResponse::internal_extension_error()
        })?;

        wasmsafe!(instance.on_request(event_queue, parts).await)
    }

    async fn on_response(
        &self,
        event_queue: Arc<EventQueue>,
        hooks_context: Arc<[u8]>,
        parts: response::Parts,
    ) -> Result<response::Parts, String> {
        let Some(pool) = self.hooks.as_ref() else {
            return Ok(parts);
        };
        let mut instance = pool.get().await.map_err(|e| e.to_string())?;

        wasmsafe!(instance.on_response(event_queue, hooks_context, parts).await)
    }
}

impl EngineHooksExtension<EngineOperationContext> for EngineWasmExtensions {
    async fn on_graphql_subgraph_request<'r>(
        &self,
        context: EngineOperationContext,
        subgraph: GraphqlSubgraph<'_>,
        parts: ReqwestParts<'r>,
    ) -> Result<ReqwestParts<'r>, GraphqlError> {
        let Some(pool) = self.gateway_extensions.hooks.as_ref() else {
            return Ok(parts);
        };
        let mut instance = pool.get().await.map_err(|e| {
            tracing::error!("Failed to get instance from pool: {e}");
            GraphqlError::internal_extension_error()
        })?;

        wasmsafe!(instance.on_graphql_subgraph_request(context, subgraph, parts).await)
    }

    async fn on_virtual_subgraph_request(
        &self,
        context: EngineOperationContext,
        subgraph: VirtualSubgraph<'_>,
        headers: http::HeaderMap,
    ) -> Result<http::HeaderMap, GraphqlError> {
        let Some(pool) = self.gateway_extensions.hooks.as_ref() else {
            return Ok(headers);
        };
        let mut instance = pool.get().await.map_err(|e| {
            tracing::error!("Failed to get instance from pool: {e}");
            GraphqlError::internal_extension_error()
        })?;

        wasmsafe!(instance.on_virtual_subgraph_request(context, subgraph, headers).await)
    }
}
