mod catalog;
mod runtime;

pub use catalog::*;
pub use runtime::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ExtensionId(u16);

pub trait Extensions: ExtensionCatalog + ExtensionRuntime {}
