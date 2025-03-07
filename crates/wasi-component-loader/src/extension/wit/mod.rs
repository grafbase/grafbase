pub mod since_0_8_0;
pub mod since_0_9_0;

pub mod directive;

// Having arguments as &[u8] is a massive pain to deal with and bindgen doesn't allow a lot of
// flexibility. Either everything is borrowed or nothing is. So wrote those manually.
pub use directive::{
    EnumDirectiveSite, FieldDefinitionDirective, FieldDefinitionDirectiveSite, InterfaceDirectiveSite,
    ObjectDirectiveSite, QueryElement, QueryElements, ScalarDirectiveSite, SchemaDirective, UnionDirectiveSite,
};

use crate::WasiState;

pub enum Extension {
    Since0_8_0(since_0_8_0::Extension),
    Since0_9_0(since_0_9_0::Extension),
}

impl Extension {
    pub(crate) fn add_to_linker(&self, linker: &mut wasmtime::component::Linker<WasiState>) -> wasmtime::Result<()> {
        match self {
            Extension::Since0_8_0(extension) => extension.add_to_linker(linker),
            Extension::Since0_9_0(extension) => extension.add_to_linker(linker),
        }
    }
}
