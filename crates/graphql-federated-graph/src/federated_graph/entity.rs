use super::{Definition, Directive, FederatedGraph, Interface, InterfaceId, Object, ObjectId, StringId};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum EntityDefinitionId {
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

pub enum EntityDefinition<'a> {
    Object(&'a Object),
    Interface(&'a Interface),
}

impl<'a> EntityDefinition<'a> {
    pub fn name(&self) -> StringId {
        match self {
            EntityDefinition::Object(obj) => obj.name,
            EntityDefinition::Interface(interface) => interface.name,
        }
    }
    pub fn directives(&'a self) -> impl Iterator<Item = &'a Directive> + 'a {
        match self {
            EntityDefinition::Object(obj) => obj.directives.iter(),
            EntityDefinition::Interface(interface) => interface.directives.iter(),
        }
    }
}

impl FederatedGraph {
    pub fn entity(&self, id: EntityDefinitionId) -> EntityDefinition<'_> {
        match id {
            EntityDefinitionId::Object(object_id) => {
                let object = &self.objects[usize::from(object_id)];
                EntityDefinition::Object(object)
            }
            EntityDefinitionId::Interface(interface_id) => {
                let interface = &self.interfaces[usize::from(interface_id)];
                EntityDefinition::Interface(interface)
            }
        }
    }
}
