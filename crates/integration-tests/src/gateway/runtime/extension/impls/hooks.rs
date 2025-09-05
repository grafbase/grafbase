use std::sync::Arc;

use engine::GraphqlError;
use engine_schema::{GraphqlSubgraph, VirtualSubgraph};
use event_queue::EventQueue;
use runtime::extension::{EngineHooksExtension, GatewayHooksExtension, OnRequest, ReqwestParts};

use crate::gateway::{EngineTestExtensions, GatewayTestExtensions};

impl GatewayHooksExtension for GatewayTestExtensions {
    async fn on_request(&self, parts: http::request::Parts) -> Result<OnRequest, engine::ErrorResponse> {
        self.wasm.on_request(parts).await
    }

    async fn on_response(
        &self,
        event_queue: Arc<EventQueue>,
        hooks_context: Arc<[u8]>,
        parts: http::response::Parts,
    ) -> Result<http::response::Parts, String> {
        self.wasm.on_response(event_queue, hooks_context, parts).await
    }
}

impl EngineHooksExtension<engine::EngineOperationContext> for EngineTestExtensions {
    async fn on_graphql_subgraph_request<'r>(
        &self,
        context: engine::EngineOperationContext,
        subgraph: GraphqlSubgraph<'_>,
        parts: ReqwestParts<'r>,
    ) -> Result<ReqwestParts<'r>, GraphqlError> {
        self.wasm.on_graphql_subgraph_request(context, subgraph, parts).await
    }

    async fn on_virtual_subgraph_request(
        &self,
        context: engine::EngineOperationContext,
        subgraph: VirtualSubgraph<'_>,
        headers: http::HeaderMap,
    ) -> Result<http::HeaderMap, GraphqlError> {
        self.wasm.on_virtual_subgraph_request(context, subgraph, headers).await
    }
}
