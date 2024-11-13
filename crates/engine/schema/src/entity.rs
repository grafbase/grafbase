use walker::{Iter, Walk};

use crate::{DefinitionId, EntityDefinition, EntityDefinitionId, TypeSystemDirective};

impl EntityDefinitionId {
    pub fn maybe_from(definition: DefinitionId) -> Option<EntityDefinitionId> {
        match definition {
            DefinitionId::Object(id) => Some(EntityDefinitionId::Object(id)),
            DefinitionId::Interface(id) => Some(EntityDefinitionId::Interface(id)),
            _ => None,
        }
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

impl PartialEq for EntityDefinition<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Interface(l), Self::Interface(r)) => l.id == r.id,
            (Self::Object(l), Self::Object(r)) => l.id == r.id,
            _ => false,
        }
    }
}

impl Eq for EntityDefinition<'_> {}
