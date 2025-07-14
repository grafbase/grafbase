use runtime::extension::{GatewayExtensions, HooksExtension};
use wasi_component_loader::extension::GatewayWasmExtensions;

use crate::gateway::ExtContext;

#[derive(Default, Clone)]
pub struct GatewayTestExtensions {
    pub wasm: GatewayWasmExtensions,
}

impl GatewayExtensions for GatewayTestExtensions {
    type Context = ExtContext;
}

impl HooksExtension<ExtContext> for GatewayTestExtensions {
    async fn on_request(
        &self,
        parts: http::request::Parts,
    ) -> Result<(ExtContext, http::request::Parts), engine::ErrorResponse> {
        let (ctx, parts) = self.wasm.on_request(parts).await?;
        let ctx = ExtContext {
            wasm: ctx,
            ..Default::default()
        };
        Ok((ctx, parts))
    }

    async fn on_response(
        &self,
        context: &ExtContext,
        parts: http::response::Parts,
    ) -> Result<http::response::Parts, String> {
        self.wasm.on_response(&context.wasm, parts).await
    }
}
