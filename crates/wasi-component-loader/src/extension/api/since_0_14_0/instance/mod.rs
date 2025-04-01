mod authentication;
mod authorization;
mod field_resolver;
mod selection_set_resolver;

use wasmtime::Store;

use crate::{Error, WasiState, extension::ExtensionInstance};

pub struct ExtensionInstanceSince0_14_0 {
    pub(crate) store: Store<WasiState>,
    pub(crate) inner: super::wit::Sdk,
    pub(crate) poisoned: bool,
}

impl ExtensionInstance for ExtensionInstanceSince0_14_0 {
    fn recycle(&mut self) -> Result<(), Error> {
        if self.poisoned {
            return Err(anyhow::anyhow!("this instance is poisoned").into());
        }

        Ok(())
    }
}
