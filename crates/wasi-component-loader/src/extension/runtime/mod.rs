mod authentication;
mod authorization;
mod field_resolver;
mod hooks;
mod resolver;
mod selection_set_resolver;
mod subscription;

use crate::{extension::GatewayWasmExtensions, resources::SharedContext};

use runtime::extension::{EngineExtensions, GatewayExtensions};

use super::EngineWasmExtensions;

impl EngineExtensions for EngineWasmExtensions {
    type Context = SharedContext;
}

impl GatewayExtensions for GatewayWasmExtensions {
    type Context = SharedContext;
}
