use id_derives::Id;
use id_newtypes::IdRange;
use schema::{
    DefinitionId, EntityDefinitionId, FieldDefinitionId, InputValueDefinitionId, InterfaceDefinitionId,
    ObjectDefinitionId, UnionDefinitionId,
};

use crate::response::{BoundResponseKey, ResponseEdge, ResponseKey};

use super::{FieldArgumentId, FieldId, Location, QueryInputValueId, SelectionSetId};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct SelectionSet {
    /// (ResponseKey, Option<FieldDefinitionId>) is guaranteed to be unique
    /// Ordered by query (parent EntityId, query position)
    pub field_ids_ordered_by_parent_entity_id_then_position: Vec<FieldId>,
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
pub enum Field {
    // Keeping attributes inside the enum to allow Rust to optimize the size of BoundField. We rarely
    // use the variants directly.
    /// __typename field
    TypeName(TypeNameField),
    /// Corresponds to an actual field within the operation that has a field definition
    Query(QueryField),
    /// Extra field added during planning to satisfy resolver/field requirements
    Extra(ExtraField),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypeNameField {
    pub type_condition: SelectionSetType,
    pub bound_response_key: BoundResponseKey,
    pub location: Location,
    pub parent_selection_set_id: SelectionSetId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryField {
    pub bound_response_key: BoundResponseKey,
    pub location: Location,
    pub definition_id: FieldDefinitionId,
    pub argument_ids: IdRange<FieldArgumentId>,
    pub selection_set_id: Option<SelectionSetId>,
    pub parent_selection_set_id: SelectionSetId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtraField {
    pub edge: ResponseEdge,
    pub definition_id: FieldDefinitionId,
    pub selection_set_id: Option<SelectionSetId>,
    pub argument_ids: IdRange<FieldArgumentId>,
    pub petitioner_location: Location,
    pub parent_selection_set_id: SelectionSetId,
}

impl Field {
    pub fn query_position(&self) -> usize {
        match self {
            Field::TypeName(TypeNameField { bound_response_key, .. }) => bound_response_key.position(),
            Field::Query(QueryField { bound_response_key, .. }) => bound_response_key.position(),
            // Fake query position, but unique
            Field::Extra(ExtraField { edge, .. }) => usize::from(*edge),
        }
    }

    pub fn response_key(&self) -> ResponseKey {
        self.response_edge()
            .as_response_key()
            .expect("BoundField don't have indices as key")
    }

    pub fn response_edge(&self) -> ResponseEdge {
        match self {
            Field::TypeName(TypeNameField { bound_response_key, .. }) => (*bound_response_key).into(),
            Field::Query(QueryField { bound_response_key, .. }) => (*bound_response_key).into(),
            Field::Extra(ExtraField { edge, .. }) => *edge,
        }
    }

    pub fn location(&self) -> Location {
        match self {
            Field::TypeName(TypeNameField { location, .. }) => *location,
            Field::Query(QueryField { location, .. }) => *location,
            Field::Extra(ExtraField {
                petitioner_location, ..
            }) => *petitioner_location,
        }
    }

    pub fn selection_set_id(&self) -> Option<SelectionSetId> {
        match self {
            Field::TypeName(TypeNameField { .. }) => None,
            Field::Query(QueryField { selection_set_id, .. }) => *selection_set_id,
            Field::Extra(ExtraField { selection_set_id, .. }) => *selection_set_id,
        }
    }

    pub fn definition_id(&self) -> Option<FieldDefinitionId> {
        match self {
            Field::TypeName(TypeNameField { .. }) => None,
            Field::Query(QueryField { definition_id, .. }) => Some(*definition_id),
            Field::Extra(ExtraField { definition_id, .. }) => Some(*definition_id),
        }
    }

    pub fn argument_ids(&self) -> IdRange<FieldArgumentId> {
        match self {
            Field::TypeName(TypeNameField { .. }) => IdRange::empty(),
            Field::Query(QueryField { argument_ids, .. }) => *argument_ids,
            Field::Extra(ExtraField { argument_ids, .. }) => *argument_ids,
        }
    }

    pub fn parent_selection_set_id(&self) -> SelectionSetId {
        match self {
            Field::TypeName(TypeNameField {
                parent_selection_set_id,
                ..
            }) => *parent_selection_set_id,
            Field::Query(QueryField {
                parent_selection_set_id,
                ..
            }) => *parent_selection_set_id,
            Field::Extra(ExtraField {
                parent_selection_set_id,
                ..
            }) => *parent_selection_set_id,
        }
    }
}

/// Represents arguments that were specified in the query with a value
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldArgument {
    pub name_location: Option<Location>,
    pub value_location: Option<Location>,
    pub input_value_definition_id: InputValueDefinitionId,
    pub input_value_id: QueryInputValueId,
}
