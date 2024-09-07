use walker::{Iter, Walk};

use crate::{DefinitionId, EntityDefinition, EntityDefinitionId, TypeSystemDirective};

impl From<EntityDefinitionId> for DefinitionId {
    fn from(value: EntityDefinitionId) -> Self {
        match value {
            EntityDefinitionId::Interface(id) => DefinitionId::Interface(id),
            EntityDefinitionId::Object(id) => DefinitionId::Object(id),
        }
    }
}

impl EntityDefinitionId {
    pub fn maybe_from(definition: DefinitionId) -> Option<EntityDefinitionId> {
        match definition {
            DefinitionId::Object(id) => Some(EntityDefinitionId::Object(id)),
            DefinitionId::Interface(id) => Some(EntityDefinitionId::Interface(id)),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, EntityDefinitionId::Object(_))
    }
}

impl<'a> EntityDefinition<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            EntityDefinition::Object(item) => item.name(),
            EntityDefinition::Interface(item) => item.name(),
        }
    }

    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        let (schema, directive_ids) = match self {
            EntityDefinition::Object(item) => (item.schema, &item.as_ref().directive_ids),
            EntityDefinition::Interface(item) => (item.schema, &item.as_ref().directive_ids),
        };
        directive_ids.walk(schema)
    }
}
