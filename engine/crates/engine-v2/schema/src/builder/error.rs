use crate::StringId;

use super::{coerce::InputValueError, BuildContext};

#[derive(Debug, Copy, Clone)]
pub enum SchemaLocation {
    Type { name: StringId },
    Field { ty: StringId, name: StringId },
}

impl SchemaLocation {
    pub fn to_string(self, ctx: &BuildContext) -> String {
        match self {
            SchemaLocation::Type { name } => ctx.strings[name].to_string(),
            SchemaLocation::Field { ty, name } => format!("{}.{}", &ctx.strings[ty], &ctx.strings[name]),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("At {location}, a required field argument is invalid. {err}")]
    RequiredFieldArgumentCoercionError { location: String, err: InputValueError },
}
