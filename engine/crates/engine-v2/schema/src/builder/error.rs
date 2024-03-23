use crate::{SchemaWalker, StringId};

use super::coerce::InputValueError;

#[derive(Debug, Copy, Clone)]
pub enum SchemaLocation {
    Type { name: StringId },
    Field { ty: StringId, name: StringId },
}

impl std::fmt::Display for SchemaWalker<'_, SchemaLocation> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.item {
            SchemaLocation::Type { name } => f.write_str(&self.schema[name]),
            SchemaLocation::Field { ty, name } => write!(f, "{}.{}", &self.schema[ty], &self.schema[name]),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    #[error("At {location}, a required field argument is invalid. {err}")]
    RequiredFieldArgumentCoercionError { location: String, err: InputValueError },
}
