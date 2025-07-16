use runtime::extension::ContractsExtension;

use crate::gateway::{EngineTestExtensions, ExtContext};

impl ContractsExtension<ExtContext> for EngineTestExtensions {
    async fn construct(
        &self,
        context: &ExtContext,
        key: String,
        schema: engine::Schema,
    ) -> Result<engine::Schema, engine::ErrorResponse> {
        self.wasm.construct(&context.wasm, key, schema).await
    }
}
