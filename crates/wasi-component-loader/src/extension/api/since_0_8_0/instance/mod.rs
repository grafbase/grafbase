mod authentication;
mod authorization;
mod field_resolver;
mod selection_set_resolver;

use wasmtime::Store;

use crate::{extension::ExtensionInstance, state::WasiState};

pub struct ExtensionInstanceSince080 {
    pub(crate) store: Store<WasiState>,
    pub(crate) inner: super::wit::Sdk,
    pub(crate) poisoned: bool,
}

impl ExtensionInstance for ExtensionInstanceSince080 {
    fn recycle(&mut self) -> crate::Result<()> {
        if self.poisoned {
            return Err(anyhow::anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }
}
