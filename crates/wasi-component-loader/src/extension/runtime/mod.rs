mod authentication;
mod authorization;
mod contracts;
mod field_resolver;
mod hooks;
mod resolver;
mod selection_set_resolver;
mod subscription;

use crate::extension::GatewayWasmExtensions;

use engine::{EngineOperationContext, EngineRequestContext};
use runtime::extension::{EngineExtensions, GatewayExtensions};

use super::EngineWasmExtensions;

impl EngineExtensions<EngineRequestContext, EngineOperationContext> for EngineWasmExtensions {}

impl GatewayExtensions for GatewayWasmExtensions {}
