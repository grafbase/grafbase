use id_newtypes::IdRange;
use schema::{Definition, EntityId, FieldDefinitionId, InputValueDefinitionId, InterfaceId, ObjectId, UnionId};

use crate::response::{BoundResponseKey, ResponseEdge, ResponseKey};

use super::{ConditionId, FieldArgumentId, FieldId, Location, QueryInputValueId, SelectionSetId};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct SelectionSet {
    /// (ResponseKey, Option<FieldDefinitionId>) is guaranteed to be unique
    pub field_ids: Vec<FieldId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SelectionSetType {
    Object(ObjectId),
    Interface(InterfaceId),
    Union(UnionId),
}

impl SelectionSetType {
    pub fn is_union(&self) -> bool {
        matches!(self, Self::Union(_))
    }
}

impl From<EntityId> for SelectionSetType {
    fn from(value: EntityId) -> Self {
        match value {
            EntityId::Object(id) => Self::Object(id),
            EntityId::Interface(id) => Self::Interface(id),
        }
    }
}

impl From<SelectionSetType> for Definition {
    fn from(parent: SelectionSetType) -> Self {
        match parent {
            SelectionSetType::Interface(id) => Self::Interface(id),
            SelectionSetType::Object(id) => Self::Object(id),
            SelectionSetType::Union(id) => Self::Union(id),
        }
    }
}

impl SelectionSetType {
    pub fn maybe_from(definition: Definition) -> Option<Self> {
        match definition {
            Definition::Object(id) => Some(SelectionSetType::Object(id)),
            Definition::Interface(id) => Some(Self::Interface(id)),
            Definition::Union(id) => Some(Self::Union(id)),
            _ => None,
        }
    }

    pub fn as_object_id(&self) -> Option<ObjectId> {
        match self {
            SelectionSetType::Object(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_entity_id(&self) -> Option<EntityId> {
        match self {
            SelectionSetType::Object(id) => Some(EntityId::Object(*id)),
            SelectionSetType::Interface(id) => Some(EntityId::Interface(*id)),
            SelectionSetType::Union(_) => None,
        }
    }
}

id_newtypes::NonZeroU16! {
    QueryPosition,
}

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
    pub field_definition_id: FieldDefinitionId,
    pub argument_ids: IdRange<FieldArgumentId>,
    pub selection_set_id: Option<SelectionSetId>,
    pub condition: Option<ConditionId>,
    pub parent_selection_set_id: SelectionSetId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtraField {
    pub edge: ResponseEdge,
    pub field_definition_id: FieldDefinitionId,
    pub selection_set_id: Option<SelectionSetId>,
    pub argument_ids: IdRange<FieldArgumentId>,
    pub petitioner_location: Location,
    pub condition: Option<ConditionId>,
    pub parent_selection_set_id: SelectionSetId,
}

impl Field {
    pub fn query_position(&self) -> usize {
        match self {
            Field::TypeName(TypeNameField { bound_response_key, .. }) => bound_response_key.position(),
            Field::Query(QueryField { bound_response_key, .. }) => bound_response_key.position(),
            Field::Extra(ExtraField { .. }) => usize::MAX,
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
            Field::Query(QueryField {
                field_definition_id, ..
            }) => Some(*field_definition_id),
            Field::Extra(ExtraField {
                field_definition_id, ..
            }) => Some(*field_definition_id),
        }
    }

    pub fn argument_ids(&self) -> IdRange<FieldArgumentId> {
        match self {
            Field::TypeName(TypeNameField { .. }) => IdRange::empty(),
            Field::Query(QueryField { argument_ids, .. }) => *argument_ids,
            Field::Extra(ExtraField { argument_ids, .. }) => *argument_ids,
        }
    }

    pub fn condition(&self) -> Option<ConditionId> {
        match self {
            Field::TypeName(TypeNameField { .. }) => None,
            Field::Query(QueryField { condition, .. }) => *condition,
            Field::Extra(ExtraField { condition, .. }) => *condition,
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
    #[allow(dead_code)]
    pub name_location: Option<Location>,
    #[allow(dead_code)]
    pub value_location: Option<Location>,
    pub input_value_definition_id: InputValueDefinitionId,
    pub input_value_id: QueryInputValueId,
}
