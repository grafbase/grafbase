use crate::StringId;

use super::{coerce::InputValueError, BuildContext};

#[derive(Debug, Copy, Clone)]
pub enum SchemaLocation {
    Definition { name_id: StringId },
    Field { ty: StringId, name_id: StringId },
}

impl SchemaLocation {
    pub fn to_string(self, ctx: &BuildContext<'_>) -> String {
        match self {
            SchemaLocation::Definition { name_id } => ctx.strings[name_id].to_string(),
            SchemaLocation::Field { ty, name_id } => format!("{}.{}", &ctx.strings[ty], &ctx.strings[name_id]),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("Invalid URL '{url}': {err}")]
    InvalidUrl { url: String, err: String },
    #[error("At {location}, a required field argument is invalid: {err}")]
    RequiredFieldArgumentCoercionError { location: String, err: InputValueError },
    #[error("An input value named '{name}' has an invalid default value: {err}")]
    DefaultValueCoercionError { name: String, err: InputValueError },
    #[error(transparent)]
    GraphFromSdlError(#[from] federated_graph::DomainError),
    #[error("Unsupported extension: {id}")]
    UnsupportedExtension { id: Box<extension_catalog::Id> },
    #[error("Could not load extension at '{url}': {err}")]
    CouldNotLoadExtension { url: String, err: String },
}
