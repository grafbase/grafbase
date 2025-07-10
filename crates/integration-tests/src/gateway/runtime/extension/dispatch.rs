use std::collections::HashMap;

use extension_catalog::ExtensionId;
use runtime::extension::EngineExtensions;
use wasi_component_loader::extension::EngineWasmExtensions;

use crate::gateway::ExtContext;

use super::TestExtensions;

#[derive(Clone, Copy)]
pub enum DispatchRule {
    Wasm,
    Test,
}

#[derive(Clone, Default)]
pub struct ExtensionsDispatcher {
    pub(super) dispatch: HashMap<ExtensionId, DispatchRule>,
    pub(super) test: TestExtensions,
    pub(super) wasm: EngineWasmExtensions,
}

impl EngineExtensions for ExtensionsDispatcher {
    type Context = ExtContext;
}
