use engine::GraphqlError;
use runtime::extension::{EngineHooksExtension, GatewayHooksExtension, OnRequest, ReqwestParts};

use crate::gateway::{EngineTestExtensions, ExtContext, GatewayTestExtensions};

impl GatewayHooksExtension<ExtContext> for GatewayTestExtensions {
    async fn on_request(&self, parts: http::request::Parts) -> Result<OnRequest<ExtContext>, engine::ErrorResponse> {
        let OnRequest {
            context,
            parts,
            contract_key,
        } = self.wasm.on_request(parts).await?;
        let ctx = ExtContext {
            wasm: context,
            ..Default::default()
        };
        Ok(OnRequest {
            context: ctx,
            parts,
            contract_key,
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
        parts: ReqwestParts,
    ) -> Result<ReqwestParts, GraphqlError> {
        self.wasm.on_graphql_subgraph_request(&context.wasm, parts).await
    }

    fn on_virtual_subgraph_request(
        &self,
        context: &ExtContext,
        subgraph: engine_schema::VirtualSubgraph<'_>,
        headers: http::HeaderMap,
    ) -> impl Future<Output = Result<http::HeaderMap, GraphqlError>> + Send {
        self.wasm
            .on_virtual_subgraph_request(&context.wasm, subgraph, headers)
            .await
    }
}
