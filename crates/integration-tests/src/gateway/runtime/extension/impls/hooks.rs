use engine::GraphqlError;
use engine_schema::{GraphqlSubgraph, VirtualSubgraph};
use runtime::extension::{
    EngineHooksExtension, ExtensionRequestContext, GatewayHooksExtension, OnRequest, ReqwestParts,
};

use crate::gateway::{EngineTestExtensions, GatewayTestExtensions};

impl GatewayHooksExtension for GatewayTestExtensions {
    async fn on_request(&self, parts: http::request::Parts) -> Result<OnRequest, engine::ErrorResponse> {
        self.wasm.on_request(parts).await
    }

    async fn on_response(
        &self,
        context: ExtensionRequestContext,
        parts: http::response::Parts,
    ) -> Result<http::response::Parts, String> {
        self.wasm.on_response(context, parts).await
    }
}

impl EngineHooksExtension<engine::EngineOperationContext> for EngineTestExtensions {
    async fn on_graphql_subgraph_request(
        &self,
        context: engine::EngineOperationContext,
        subgraph: GraphqlSubgraph<'_>,
        parts: ReqwestParts,
    ) -> Result<ReqwestParts, GraphqlError> {
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
