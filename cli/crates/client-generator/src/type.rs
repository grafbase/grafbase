mod condition;
mod generator;
mod identifier;
mod kind;
mod mapped;
mod name;
mod object;
mod property;
mod r#static;

use std::fmt;

pub use condition::TypeCondition;
pub use generator::TypeGenerator;
pub use identifier::TypeIdentifier;
pub use kind::TypeKind;
pub use mapped::{MappedType, TypeMapSource};
pub use name::TypeName;
pub use object::ObjectTypeDef;
pub use property::{Property, PropertyValue};
pub use r#static::StaticType;

#[derive(Debug)]
pub struct Type {
    identifier: StaticType,
    definition: TypeKind,
}

impl Type {
    pub fn new(identifier: StaticType, definition: impl Into<TypeKind>) -> Self {
        Self {
            identifier,
            definition: definition.into(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {} = {}", self.identifier, self.definition)
    }
}
