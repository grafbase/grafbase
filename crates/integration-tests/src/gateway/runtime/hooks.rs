use std::collections::HashMap;

use extension_catalog::ExtensionId;
use runtime::extension::GatewayExtensions;
use wasi_component_loader::extension::GatewayWasmExtensions;

use crate::gateway::{DispatchRule, TestExtensions};

#[derive(Default, Clone)]
pub struct GatewayTestExtensions {
    pub dispatch: HashMap<ExtensionId, DispatchRule>,
    pub wasm: GatewayWasmExtensions,
    pub test: TestExtensions,
}

impl GatewayExtensions for GatewayTestExtensions {}
