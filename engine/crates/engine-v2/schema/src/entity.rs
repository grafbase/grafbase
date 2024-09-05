use crate::{DefinitionId, EntityDefinitionId};

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
