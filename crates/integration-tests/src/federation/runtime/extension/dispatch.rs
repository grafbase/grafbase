use std::collections::HashMap;

use extension_catalog::ExtensionId;
use runtime::extension::ExtensionRuntime;
use wasi_component_loader::extension::WasmExtensions;

use crate::federation::ExtContext;

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
    pub(super) wasm: WasmExtensions,
}

impl ExtensionRuntime for ExtensionsDispatcher {
    type Context = ExtContext;
}
