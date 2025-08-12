use engine::GraphqlError;
use engine_schema::{GraphqlSubgraph, VirtualSubgraph};
use runtime::extension::{EngineHooksExtension, GatewayHooksExtension, OnRequest, ReqwestParts};

use crate::gateway::{EngineTestExtensions, ExtContext, GatewayTestExtensions};

impl GatewayHooksExtension<ExtContext> for GatewayTestExtensions {
    async fn on_request(&self, parts: http::request::Parts) -> Result<OnRequest<ExtContext>, engine::ErrorResponse> {
        let OnRequest {
            context,
            parts,
            contract_key,
            context: state,
        } = self.wasm.on_request(parts).await?;
        let ctx = ExtContext {
            wasm: context,
            ..Default::default()
        };
        Ok(OnRequest {
            context: ctx,
            parts,
            contract_key,
            context: state,
        })
    }

    async fn on_response(
        &self,
        context: ExtContext,
        parts: http::response::Parts,
    ) -> Result<http::response::Parts, String> {
        self.wasm.on_response(context.wasm, parts).await
    }
}

impl EngineHooksExtension<ExtContext> for EngineTestExtensions {
    async fn on_graphql_subgraph_request(
        &self,
        context: &ExtContext,
        subgraph: GraphqlSubgraph<'_>,
        parts: ReqwestParts,
    ) -> Result<ReqwestParts, GraphqlError> {
        self.wasm
            .on_graphql_subgraph_request(&context.wasm, subgraph, parts)
            .await
    }

    async fn on_virtual_subgraph_request(
        &self,
        context: &ExtContext,
        subgraph: VirtualSubgraph<'_>,
        headers: http::HeaderMap,
    ) -> Result<http::HeaderMap, GraphqlError> {
        self.wasm
            .on_virtual_subgraph_request(&context.wasm, subgraph, headers)
            .await
    }
}
