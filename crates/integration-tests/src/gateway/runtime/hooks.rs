use runtime::extension::HooksExtension;
use wasi_component_loader::extension::GatewayWasmExtensions;

use crate::gateway::ExtContext;

#[derive(Default, Clone)]
pub struct GatewayTestExtensions {
    pub wasm: GatewayWasmExtensions,
}

impl HooksExtension for GatewayTestExtensions {
    type Context = ExtContext;

    fn new_context(&self) -> Self::Context {
        ExtContext {
            wasm: self.wasm.new_context(),
            ..Default::default()
        }
    }

    async fn on_request(
        &self,
        context: &Self::Context,
        parts: http::request::Parts,
    ) -> Result<http::request::Parts, engine::ErrorResponse> {
        self.wasm.on_request(&context.wasm, parts).await
    }

    async fn on_response(
        &self,
        context: &Self::Context,
        parts: http::response::Parts,
    ) -> Result<http::response::Parts, String> {
        self.wasm.on_response(&context.wasm, parts).await
    }
}
