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

pub struct Type<'a> {
    identifier: StaticType<'a>,
    definition: TypeKind<'a>,
}

impl<'a> Type<'a> {
    pub fn new(identifier: StaticType<'a>, definition: impl Into<TypeKind<'a>>) -> Self {
        Self {
            identifier,
            definition: definition.into(),
        }
    }
}

impl<'a> fmt::Display for Type<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type {} = {}", self.identifier, self.definition)
    }
}
