mod authentication;
mod authorization;
mod field_resolver;
mod hooks;
mod resolver;
mod selection_set_resolver;
mod subscription;

use crate::resources::SharedContext;

use runtime::extension::ExtensionRuntime;

use super::WasmExtensions;

impl ExtensionRuntime for WasmExtensions {
    type Context = SharedContext;
}
