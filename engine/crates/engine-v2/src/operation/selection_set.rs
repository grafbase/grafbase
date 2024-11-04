use id_derives::Id;
use id_newtypes::IdRange;
use schema::{
    DefinitionId, EntityDefinitionId, FieldDefinitionId, InputValueDefinitionId, InterfaceDefinitionId,
    ObjectDefinitionId, UnionDefinitionId,
};

use crate::response::{BoundResponseKey, ResponseEdge, ResponseKey};

use super::{BoundFieldArgumentId, BoundFieldId, BoundSelectionSetId, Location, QueryInputValueId};

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct BoundSelectionSet {
    /// (ResponseKey, Option<FieldDefinitionId>) is guaranteed to be unique
    /// Ordered by query (parent EntityId, query position)
    pub field_ids_ordered_by_parent_entity_id_then_position: Vec<BoundFieldId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum SelectionSetType {
    Object(ObjectDefinitionId),
    Interface(InterfaceDefinitionId),
    Union(UnionDefinitionId),
}

impl SelectionSetType {
    pub fn is_union(&self) -> bool {
        matches!(self, Self::Union(_))
    }
}

impl From<EntityDefinitionId> for SelectionSetType {
    fn from(value: EntityDefinitionId) -> Self {
        match value {
            EntityDefinitionId::Object(id) => Self::Object(id),
            EntityDefinitionId::Interface(id) => Self::Interface(id),
        }
    }
}

impl From<SelectionSetType> for DefinitionId {
    fn from(parent: SelectionSetType) -> Self {
        match parent {
            SelectionSetType::Interface(id) => Self::Interface(id),
            SelectionSetType::Object(id) => Self::Object(id),
            SelectionSetType::Union(id) => Self::Union(id),
        }
    }
}

impl SelectionSetType {
    pub fn maybe_from(definition: DefinitionId) -> Option<Self> {
        match definition {
            DefinitionId::Object(id) => Some(SelectionSetType::Object(id)),
            DefinitionId::Interface(id) => Some(Self::Interface(id)),
            DefinitionId::Union(id) => Some(Self::Union(id)),
            _ => None,
        }
    }

    pub fn as_object_id(&self) -> Option<ObjectDefinitionId> {
        match self {
            SelectionSetType::Object(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_entity_id(&self) -> Option<EntityDefinitionId> {
        match self {
            SelectionSetType::Object(id) => Some(EntityDefinitionId::Object(*id)),
            SelectionSetType::Interface(id) => Some(EntityDefinitionId::Interface(*id)),
            SelectionSetType::Union(_) => None,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Id)]
pub struct QueryPosition(std::num::NonZero<u16>);

/// The BoundFieldDefinition defines a field that is part of the actual GraphQL query.
/// A BoundField is a field in the query *after* spreading all the named fragments.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BoundField {
    // Keeping attributes inside the enum to allow Rust to optimize the size of BoundField. We rarely
    // use the variants directly.
    /// __typename field
    TypeName(BoundTypeNameField),
    /// Corresponds to an actual field within the operation that has a field definition
    Query(BoundQueryField),
    /// Extra field added during planning to satisfy resolver/field requirements
    Extra(BoundExtraField),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoundTypeNameField {
    pub type_condition: SelectionSetType,
    pub bound_response_key: BoundResponseKey,
    pub location: Location,
    pub parent_selection_set_id: BoundSelectionSetId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoundQueryField {
    pub bound_response_key: BoundResponseKey,
    pub location: Location,
    pub definition_id: FieldDefinitionId,
    pub argument_ids: IdRange<BoundFieldArgumentId>,
    pub selection_set_id: Option<BoundSelectionSetId>,
    pub parent_selection_set_id: BoundSelectionSetId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoundExtraField {
    pub edge: ResponseEdge,
    pub definition_id: FieldDefinitionId,
    pub selection_set_id: Option<BoundSelectionSetId>,
    pub argument_ids: IdRange<BoundFieldArgumentId>,
    pub petitioner_location: Location,
    pub parent_selection_set_id: BoundSelectionSetId,
}

impl BoundField {
    pub fn query_position(&self) -> usize {
        match self {
            BoundField::TypeName(BoundTypeNameField { bound_response_key, .. }) => bound_response_key.position(),
            BoundField::Query(BoundQueryField { bound_response_key, .. }) => bound_response_key.position(),
            // Fake query position, but unique
            BoundField::Extra(BoundExtraField { edge, .. }) => usize::from(*edge),
        }
    }

    pub fn response_key(&self) -> ResponseKey {
        self.response_edge()
            .as_response_key()
            .expect("BoundField don't have indices as key")
    }

    pub fn response_edge(&self) -> ResponseEdge {
        match self {
            BoundField::TypeName(BoundTypeNameField { bound_response_key, .. }) => (*bound_response_key).into(),
            BoundField::Query(BoundQueryField { bound_response_key, .. }) => (*bound_response_key).into(),
            BoundField::Extra(BoundExtraField { edge, .. }) => *edge,
        }
    }

    pub fn location(&self) -> Location {
        match self {
            BoundField::TypeName(BoundTypeNameField { location, .. }) => *location,
            BoundField::Query(BoundQueryField { location, .. }) => *location,
            BoundField::Extra(BoundExtraField {
                petitioner_location, ..
            }) => *petitioner_location,
        }
    }

    pub fn selection_set_id(&self) -> Option<BoundSelectionSetId> {
        match self {
            BoundField::TypeName(BoundTypeNameField { .. }) => None,
            BoundField::Query(BoundQueryField { selection_set_id, .. }) => *selection_set_id,
            BoundField::Extra(BoundExtraField { selection_set_id, .. }) => *selection_set_id,
        }
    }

    pub fn definition_id(&self) -> Option<FieldDefinitionId> {
        match self {
            BoundField::TypeName(BoundTypeNameField { .. }) => None,
            BoundField::Query(BoundQueryField { definition_id, .. }) => Some(*definition_id),
            BoundField::Extra(BoundExtraField { definition_id, .. }) => Some(*definition_id),
        }
    }

    pub fn argument_ids(&self) -> IdRange<BoundFieldArgumentId> {
        match self {
            BoundField::TypeName(BoundTypeNameField { .. }) => IdRange::empty(),
            BoundField::Query(BoundQueryField { argument_ids, .. }) => *argument_ids,
            BoundField::Extra(BoundExtraField { argument_ids, .. }) => *argument_ids,
        }
    }

    pub fn parent_selection_set_id(&self) -> BoundSelectionSetId {
        match self {
            BoundField::TypeName(BoundTypeNameField {
                parent_selection_set_id,
                ..
            }) => *parent_selection_set_id,
            BoundField::Query(BoundQueryField {
                parent_selection_set_id,
                ..
            }) => *parent_selection_set_id,
            BoundField::Extra(BoundExtraField {
                parent_selection_set_id,
                ..
            }) => *parent_selection_set_id,
        }
    }
}

/// Represents arguments that were specified in the query with a value
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BoundFieldArgument {
    pub name_location: Option<Location>,
    pub value_location: Option<Location>,
    pub input_value_definition_id: InputValueDefinitionId,
    pub input_value_id: QueryInputValueId,
}
