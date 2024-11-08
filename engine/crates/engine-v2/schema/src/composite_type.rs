use crate::{
    CompositeType, CompositeTypeId, EntityDefinition, EntityDefinitionId, InterfaceDefinitionId, ObjectDefinitionId,
    UnionDefinitionId,
};

impl From<EntityDefinitionId> for CompositeTypeId {
    fn from(value: EntityDefinitionId) -> Self {
        match value {
            EntityDefinitionId::Interface(id) => CompositeTypeId::Interface(id),
            EntityDefinitionId::Object(id) => CompositeTypeId::Object(id),
        }
    }
}

impl<'a> From<EntityDefinition<'a>> for CompositeType<'a> {
    fn from(value: EntityDefinition<'a>) -> Self {
        match value {
            EntityDefinition::Interface(def) => CompositeType::Interface(def),
            EntityDefinition::Object(def) => CompositeType::Object(def),
        }
    }
}

impl CompositeTypeId {
    pub fn as_entity(&self) -> Option<EntityDefinitionId> {
        match self {
            CompositeTypeId::Interface(id) => Some(EntityDefinitionId::Interface(*id)),
            CompositeTypeId::Object(id) => Some(EntityDefinitionId::Object(*id)),
            CompositeTypeId::Union(_) => None,
        }
    }

    pub fn as_object(&self) -> Option<ObjectDefinitionId> {
        match self {
            CompositeTypeId::Object(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_interface(&self) -> Option<InterfaceDefinitionId> {
        match self {
            CompositeTypeId::Interface(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_union(&self) -> Option<UnionDefinitionId> {
        match self {
            CompositeTypeId::Union(id) => Some(*id),
            _ => None,
        }
    }
}
