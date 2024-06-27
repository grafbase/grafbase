use std::borrow::Cow;

use id_newtypes::IdRange;
use schema::{Definition, EntityId, FieldDefinitionId, InputValueDefinitionId, InterfaceId, ObjectId, Schema, UnionId};

use crate::response::{BoundResponseKey, ResponseEdge, ResponseKey};

use super::{
    ConditionId, FieldArgumentId, FieldId, FragmentId, FragmentSpreadId, InlineFragmentId, Location, QueryInputValueId,
    SelectionSetId,
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SelectionSet {
    pub ty: SelectionSetType,
    // Ordering matters and must be respected in the response.
    pub items: Vec<Selection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl From<SelectionSetType> for TypeCondition {
    fn from(parent: SelectionSetType) -> Self {
        match parent {
            SelectionSetType::Interface(id) => Self::Interface(id),
            SelectionSetType::Object(id) => Self::Object(id),
            SelectionSetType::Union(id) => Self::Union(id),
        }
    }
}

impl From<TypeCondition> for SelectionSetType {
    fn from(cond: TypeCondition) -> Self {
        match cond {
            TypeCondition::Interface(id) => Self::Interface(id),
            TypeCondition::Object(id) => Self::Object(id),
            TypeCondition::Union(id) => Self::Union(id),
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

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Selection {
    Field(FieldId),
    FragmentSpread(FragmentSpreadId),
    InlineFragment(InlineFragmentId),
}

/// The BoundFieldDefinition defines a field that is part of the actual GraphQL query.
/// A BoundField is a field in the query *after* spreading all the named fragments.
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct TypeNameField {
    pub bound_response_key: BoundResponseKey,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub struct QueryField {
    pub bound_response_key: BoundResponseKey,
    pub location: Location,
    pub field_definition_id: FieldDefinitionId,
    pub argument_ids: IdRange<FieldArgumentId>,
    pub selection_set_id: Option<SelectionSetId>,
    pub condition: Option<ConditionId>,
}

#[derive(Debug, Clone)]
pub struct ExtraField {
    pub edge: ResponseEdge,
    pub field_definition_id: FieldDefinitionId,
    pub selection_set_id: Option<SelectionSetId>,
    pub argument_ids: IdRange<FieldArgumentId>,
    pub petitioner_location: Location,
    // FIXME: Could probably avoid having those by having those additional extra fields be in a
    // temporary struct instead.
    /// During the planning we may add more extra fields than necessary. To prevent retrieving
    /// unnecessary data, only those marked as read are part of the operation.
    pub is_read: bool,
    pub condition: Option<ConditionId>,
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

    pub fn mark_as_read(&mut self) {
        match self {
            Field::TypeName(TypeNameField { .. }) => (),
            Field::Query(QueryField { .. }) => (),
            Field::Extra(ExtraField { is_read, .. }) => *is_read = true,
        }
    }

    pub fn is_read(&self) -> bool {
        match self {
            Field::TypeName(TypeNameField { .. }) => true,
            Field::Query(QueryField { .. }) => true,
            Field::Extra(ExtraField { is_read, .. }) => *is_read,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentSpread {
    pub location: Location,
    pub fragment_id: FragmentId,
    // This selection set is bound to its actual position in the query.
    pub selection_set_id: SelectionSetId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineFragment {
    pub location: Location,
    pub type_condition: Option<TypeCondition>,
    pub selection_set_id: SelectionSetId,
}

#[derive(Debug, Clone)]
pub struct Fragment {
    pub name: String,
    pub name_location: Location,
    pub type_condition: TypeCondition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeCondition {
    Interface(InterfaceId),
    Object(ObjectId),
    Union(UnionId),
}

impl TypeCondition {
    pub fn resolve(self, schema: &Schema) -> Cow<'_, Vec<ObjectId>> {
        match self {
            TypeCondition::Interface(interface_id) => Cow::Borrowed(&schema[interface_id].possible_types),
            TypeCondition::Object(object_id) => Cow::Owned(vec![object_id]),
            TypeCondition::Union(union_id) => Cow::Borrowed(&schema[union_id].possible_types),
        }
    }
}

impl From<TypeCondition> for Definition {
    fn from(value: TypeCondition) -> Self {
        match value {
            TypeCondition::Interface(id) => Definition::Interface(id),
            TypeCondition::Object(id) => Definition::Object(id),
            TypeCondition::Union(id) => Definition::Union(id),
        }
    }
}

/// Represents arguments that were specified in the query with a value
#[derive(Debug, Clone)]
pub struct FieldArgument {
    pub name_location: Option<Location>,
    pub value_location: Option<Location>,
    pub input_value_definition_id: InputValueDefinitionId,
    pub input_value_id: QueryInputValueId,
}

impl IntoIterator for SelectionSet {
    type Item = Selection;

    type IntoIter = <Vec<Selection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a SelectionSet {
    type Item = &'a Selection;

    type IntoIter = <&'a Vec<Selection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
