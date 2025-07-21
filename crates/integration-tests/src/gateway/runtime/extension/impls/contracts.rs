use runtime::extension::ContractsExtension;

use crate::gateway::{EngineTestExtensions, ExtContext};

impl ContractsExtension<ExtContext> for EngineTestExtensions {
    async fn construct(&self, context: &ExtContext, key: String, schema: engine::Schema) -> Option<engine::Schema> {
        self.wasm.construct(&context.wasm, key, schema).await
    }
}
