mod authentication;
mod authorization;
mod contracts;
mod field_resolver;
mod hooks;
mod resolver;
mod selection_set_resolver;
mod subscription;

use crate::{extension::GatewayWasmExtensions, resources::WasmContext};

use runtime::extension::{EngineExtensions, GatewayExtensions};

use super::EngineWasmExtensions;

impl EngineExtensions for EngineWasmExtensions {}

impl GatewayExtensions for GatewayWasmExtensions {}
