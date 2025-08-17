use std::{collections::HashMap, sync::Arc};

use engine::Schema;
use extension_catalog::ExtensionId;
use runtime::extension::EngineExtensions;
use wasi_component_loader::extension::EngineWasmExtensions;

use super::TestExtensions;

#[derive(Clone, Copy)]
pub enum DispatchRule {
    Wasm,
    Test,
}

#[derive(Clone, Default)]
pub struct EngineTestExtensions {
    pub(super) dispatch: HashMap<ExtensionId, DispatchRule>,
    pub(super) test: TestExtensions,
    pub(super) wasm: EngineWasmExtensions,
}

impl EngineExtensions<engine::EngineRequestContext, engine::EngineOperationContext> for EngineTestExtensions {}

impl EngineTestExtensions {
    pub async fn clone_and_adjust_for_contract(&self, schema: &Arc<Schema>) -> Result<Self, String> {
        Ok(Self {
            dispatch: self.dispatch.clone(),
            test: self.test.clone(),
            wasm: self
                .wasm
                .clone_and_adjust_for_contract(schema)
                .await
                .map_err(|err| err.to_string())?,
        })
    }
}
