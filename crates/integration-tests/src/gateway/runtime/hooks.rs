use runtime::extension::GatewayExtensions;
use wasi_component_loader::extension::GatewayWasmExtensions;

use crate::gateway::{ExtContext, TestExtensions};

#[derive(Default, Clone)]
pub struct GatewayTestExtensions {
    pub wasm: GatewayWasmExtensions,
    pub test: TestExtensions,
}

impl GatewayExtensions for GatewayTestExtensions {
    type Context = ExtContext;
}
