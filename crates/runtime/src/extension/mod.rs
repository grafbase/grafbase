mod catalog;
mod runtime;

pub use catalog::*;
pub use runtime::*;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ExtensionId(u16);
