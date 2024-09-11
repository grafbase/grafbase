use walker::Walk;

use crate::{Definition, EntityDefinition, ScalarType, TypeSystemDirective};

impl<'a> Definition<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            Definition::Enum(item) => item.name(),
            Definition::InputObject(item) => item.name(),
            Definition::Interface(item) => item.name(),
            Definition::Object(item) => item.name(),
            Definition::Scalar(item) => item.name(),
            Definition::Union(item) => item.name(),
        }
    }

    pub fn directives(&self) -> impl Iterator<Item = TypeSystemDirective<'a>> + 'a {
        let (schema, directive_ids) = match self {
            Definition::Enum(item) => (item.schema, &item.as_ref().directive_ids),
            Definition::InputObject(item) => (item.schema, &item.as_ref().directive_ids),
            Definition::Interface(item) => (item.schema, &item.as_ref().directive_ids),
            Definition::Object(item) => (item.schema, &item.as_ref().directive_ids),
            Definition::Scalar(item) => (item.schema, &item.as_ref().directive_ids),
            Definition::Union(item) => (item.schema, &item.as_ref().directive_ids),
        };
        directive_ids.walk(schema)
    }

    pub fn scalar_type(&self) -> Option<ScalarType> {
        match self {
            Definition::Scalar(scalar) => Some(scalar.ty),
            _ => None,
        }
    }

    pub fn as_entity(&self) -> Option<EntityDefinition<'a>> {
        match self {
            Definition::Object(object) => Some(EntityDefinition::Object(*object)),
            Definition::Interface(interface) => Some(EntityDefinition::Interface(*interface)),
            _ => None,
        }
    }
}
