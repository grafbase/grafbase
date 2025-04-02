use walker::Walk;

use crate::{
    CompositeType, CompositeTypeId, EntityDefinition, EntityDefinitionId, ObjectDefinitionId, SubgraphId,
    TypeDefinition, TypeDefinitionId,
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
    pub fn maybe_from(id: TypeDefinitionId) -> Option<Self> {
        match id {
            TypeDefinitionId::Interface(id) => Some(CompositeTypeId::Interface(id)),
            TypeDefinitionId::Object(id) => Some(CompositeTypeId::Object(id)),
            TypeDefinitionId::Union(id) => Some(CompositeTypeId::Union(id)),
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
    pub fn name(&self) -> &'a str {
        match self {
            CompositeType::Interface(def) => def.name(),
            CompositeType::Object(def) => def.name(),
            CompositeType::Union(def) => def.name(),
        }
    }

    pub fn maybe_from(definition: TypeDefinition<'a>) -> Option<Self> {
        match definition {
            TypeDefinition::Interface(def) => Some(CompositeType::Interface(def)),
            TypeDefinition::Object(def) => Some(CompositeType::Object(def)),
            TypeDefinition::Union(def) => Some(CompositeType::Union(def)),
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

    pub fn has_inaccessible_possible_type(&self) -> bool {
        match self {
            CompositeType::Union(def) => def.has_inaccessible_member(),
            CompositeType::Interface(def) => def.has_inaccessible_implementor(),
            CompositeType::Object(def) => def.is_inaccessible(),
        }
    }

    pub fn is_fully_implemented_in_subgraph(&self, id: SubgraphId) -> bool {
        match self {
            CompositeType::Interface(def) => def.is_fully_implemented_in(id),
            CompositeType::Union(def) => def.is_fully_implemented_in(id),
            CompositeType::Object(def) => def.exists_in_subgraph(&id),
        }
    }

    pub fn possible_types_include_in_subgraph(&self, subgraph_id: SubgraphId, object_id: ObjectDefinitionId) -> bool {
        match self {
            CompositeType::Interface(interface) => object_id
                .walk(interface.schema)
                .implements_interface_in_subgraph(&subgraph_id, &interface.id),
            CompositeType::Union(union) => union.has_member_in_subgraph(subgraph_id, object_id),
            CompositeType::Object(object) => object.id == object_id && object.exists_in_subgraph(&subgraph_id),
        }
    }

    pub fn possible_type_ids(&self) -> &[ObjectDefinitionId] {
        match self {
            CompositeType::Object(object) => std::array::from_ref(&object.id),
            CompositeType::Interface(interface) => &interface.possible_type_ids,
            CompositeType::Union(union) => &union.possible_type_ids,
        }
    }

    pub fn has_non_empty_intersection_with(&self, other: CompositeType<'a>) -> bool {
        let left = self.possible_type_ids();
        let right = other.possible_type_ids();
        let mut l = 0;
        let mut r = 0;
        while let (Some(left_id), Some(right_id)) = (left.get(l), right.get(r)) {
            match left_id.cmp(right_id) {
                std::cmp::Ordering::Less => l += 1,
                // At least one common object
                std::cmp::Ordering::Equal => return true,
                std::cmp::Ordering::Greater => r += 1,
            }
        }
        false
    }

    pub fn is_subset_of(&self, other: CompositeType<'a>) -> bool {
        let subset = self.possible_type_ids();
        let superset = other.possible_type_ids();
        if subset.len() > superset.len() {
            return false;
        }

        let mut sub_i = 0;
        let mut super_i = 0;
        while let (Some(sub_id), Some(super_id)) = (subset.get(sub_i), superset.get(super_i)) {
            match sub_id.cmp(super_id) {
                // Cannot exist in superset, so not a superset.
                std::cmp::Ordering::Less => return false,
                std::cmp::Ordering::Equal => sub_i += 1,
                std::cmp::Ordering::Greater => super_i += 1,
            }
        }
        true
    }
}
