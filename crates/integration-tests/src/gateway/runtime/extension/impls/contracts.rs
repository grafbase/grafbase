use runtime::extension::ContractsExtension;

use crate::gateway::EngineTestExtensions;

impl ContractsExtension for EngineTestExtensions {
    async fn construct(&self, key: String, schema: engine::Schema) -> Option<engine::Schema> {
        self.wasm.construct(key, schema).await
    }
}
