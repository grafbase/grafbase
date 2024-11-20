use walker::Walk;

use crate::{
    CompositeType, CompositeTypeId, Definition, DefinitionId, EntityDefinition, EntityDefinitionId, ObjectDefinitionId,
    SubgraphId,
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
    pub fn maybe_from(id: DefinitionId) -> Option<Self> {
        match id {
            DefinitionId::Interface(id) => Some(CompositeTypeId::Interface(id)),
            DefinitionId::Object(id) => Some(CompositeTypeId::Object(id)),
            DefinitionId::Union(id) => Some(CompositeTypeId::Union(id)),
            _ => None,
        }
    }

    pub fn as_entity(&self) -> Option<EntityDefinitionId> {
        match self {
            CompositeTypeId::Interface(id) => Some(EntityDefinitionId::Interface(*id)),
            CompositeTypeId::Object(id) => Some(EntityDefinitionId::Object(*id)),
            CompositeTypeId::Union(_) => None,
        }
    }
}

impl<'a> CompositeType<'a> {
    pub fn maybe_from(definition: Definition<'a>) -> Option<Self> {
        match definition {
            Definition::Interface(def) => Some(CompositeType::Interface(def)),
            Definition::Object(def) => Some(CompositeType::Object(def)),
            Definition::Union(def) => Some(CompositeType::Union(def)),
            _ => None,
        }
    }

    pub fn as_entity(&self) -> Option<EntityDefinition<'a>> {
        match self {
            CompositeType::Interface(def) => Some(EntityDefinition::Interface(*def)),
            CompositeType::Object(def) => Some(EntityDefinition::Object(*def)),
            CompositeType::Union(_) => None,
        }
    }

    pub fn is_fully_implemented_in_subgraph(&self, id: SubgraphId) -> bool {
        match self {
            CompositeType::Interface(def) => def.is_fully_implemented_in(id),
            CompositeType::Union(def) => def.is_fully_implemented_in(id),
            CompositeType::Object(def) => def.is_resolvable_in(&id),
        }
    }

    pub fn possible_types_include_in_subgraph(&self, subgraph_id: SubgraphId, object_id: ObjectDefinitionId) -> bool {
        match self {
            CompositeType::Interface(interface) => object_id
                .walk(interface.schema)
                .implements_interface_in_subgraph(&subgraph_id, &interface.id),
            CompositeType::Union(union) => union.has_member_in_subgraph(subgraph_id, object_id),
            CompositeType::Object(object) => object.id == object_id && object.is_resolvable_in(&subgraph_id),
        }
    }
}
