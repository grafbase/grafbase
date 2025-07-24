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
    async fn on_subgraph_request(
        &self,
        context: &ExtContext,
        parts: ReqwestParts,
    ) -> Result<ReqwestParts, GraphqlError> {
        self.wasm.on_subgraph_request(&context.wasm, parts).await
    }
}
