use super::{Definition, InterfaceId, ObjectId};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum EntityDefinitionId {
    Object(ObjectId),
    Interface(InterfaceId),
}

impl From<EntityDefinitionId> for Definition {
    fn from(entity_definition_id: EntityDefinitionId) -> Self {
        match entity_definition_id {
            EntityDefinitionId::Object(object_id) => Definition::Object(object_id),
            EntityDefinitionId::Interface(interface_id) => Definition::Interface(interface_id),
        }
    }
}

impl From<ObjectId> for EntityDefinitionId {
    fn from(object_id: ObjectId) -> Self {
        EntityDefinitionId::Object(object_id)
    }
}

impl From<InterfaceId> for EntityDefinitionId {
    fn from(interface_id: InterfaceId) -> Self {
        EntityDefinitionId::Interface(interface_id)
    }
}
