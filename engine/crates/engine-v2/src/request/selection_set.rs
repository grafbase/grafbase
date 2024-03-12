use std::borrow::Cow;

use id_newtypes::IdRange;
use schema::{Definition, FieldId, InputValueDefinitionId, InterfaceId, ObjectId, Schema, UnionId};

use crate::response::{BoundResponseKey, ResponseEdge, ResponseKey};

use super::{
    BoundFieldArgumentId, BoundFieldId, BoundFragmentId, BoundFragmentSpreadId, BoundInlineFragmentId,
    BoundSelectionSetId, Location, OpInputValueId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundSelectionSet {
    pub ty: SelectionSetType,
    // Ordering matters and must be respected in the response.
    pub items: Vec<BoundSelection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectionSetType {
    Object(ObjectId),
    Interface(InterfaceId),
    Union(UnionId),
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BoundSelection {
    Field(BoundFieldId),
    FragmentSpread(BoundFragmentSpreadId),
    InlineFragment(BoundInlineFragmentId),
}

/// The BoundFieldDefinition defines a field that is part of the actual GraphQL query.
/// A BoundField is a field in the query *after* spreading all the named fragments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundField {
    // Keeping attributes inside the enum to allow Rust to optimize the size of BoundField. We rarely
    // use the variants directly.
    /// __typename field
    TypeName {
        bound_response_key: BoundResponseKey,
        location: Location,
    },
    /// Corresponds to an actual field within the operation
    Field {
        bound_response_key: BoundResponseKey,
        location: Location,
        field_id: FieldId,
        argument_ids: IdRange<BoundFieldArgumentId>,
        selection_set_id: Option<BoundSelectionSetId>,
    },
    /// Extra field added during planning to satisfy resolver/field requirements
    Extra {
        edge: ResponseEdge,
        field_id: FieldId,
        selection_set_id: Option<BoundSelectionSetId>,
        /// During the planning we may add more extra fields than necessary. To prevent retrieving
        /// unecessary data, only those marked as read are part of the opeartion.
        is_read: bool,
    },
}

impl BoundField {
    pub fn query_position(&self) -> usize {
        match self {
            BoundField::TypeName { bound_response_key, .. } => bound_response_key.position(),
            BoundField::Field { bound_response_key, .. } => bound_response_key.position(),
            BoundField::Extra { .. } => usize::MAX,
        }
    }

    pub fn response_key(&self) -> ResponseKey {
        self.response_edge()
            .as_response_key()
            .expect("BoundField don't have indices as key")
    }

    pub fn response_edge(&self) -> ResponseEdge {
        match self {
            BoundField::TypeName { bound_response_key, .. } => (*bound_response_key).into(),
            BoundField::Field { bound_response_key, .. } => (*bound_response_key).into(),
            BoundField::Extra { edge, .. } => *edge,
        }
    }

    pub fn name_location(&self) -> Option<Location> {
        match self {
            BoundField::TypeName { location, .. } => Some(*location),
            BoundField::Field { location, .. } => Some(*location),
            BoundField::Extra { .. } => None,
        }
    }

    pub fn selection_set_id(&self) -> Option<BoundSelectionSetId> {
        match self {
            BoundField::TypeName { .. } => None,
            BoundField::Field { selection_set_id, .. } => *selection_set_id,
            BoundField::Extra { selection_set_id, .. } => *selection_set_id,
        }
    }

    pub fn schema_field_id(&self) -> Option<FieldId> {
        match self {
            BoundField::TypeName { .. } => None,
            BoundField::Field { field_id, .. } => Some(*field_id),
            BoundField::Extra { field_id, .. } => Some(*field_id),
        }
    }

    pub fn mark_as_read(&mut self) {
        match self {
            BoundField::TypeName { .. } => {}
            BoundField::Field { .. } => {}
            BoundField::Extra { is_read, .. } => *is_read = true,
        }
    }

    pub fn is_read(&self) -> bool {
        match self {
            BoundField::TypeName { .. } => true,
            BoundField::Field { .. } => true,
            BoundField::Extra { is_read, .. } => *is_read,
        }
    }

    pub fn argument_ids(&self) -> IdRange<BoundFieldArgumentId> {
        match self {
            BoundField::TypeName { .. } => IdRange::empty(),
            BoundField::Field { argument_ids, .. } => *argument_ids,
            BoundField::Extra { .. } => IdRange::empty(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundFragmentSpread {
    pub location: Location,
    pub fragment_id: BoundFragmentId,
    // This selection set is bound to its actual position in the query.
    pub selection_set_id: BoundSelectionSetId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundInlineFragment {
    pub location: Location,
    pub type_condition: Option<TypeCondition>,
    pub selection_set_id: BoundSelectionSetId,
}

#[derive(Debug)]
pub struct BoundFragment {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundFieldArgument {
    pub name_location: Option<Location>,
    pub value_location: Option<Location>,
    pub input_value_definition_id: InputValueDefinitionId,
    pub input_value_id: OpInputValueId,
}

impl IntoIterator for BoundSelectionSet {
    type Item = BoundSelection;

    type IntoIter = <Vec<BoundSelection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a BoundSelectionSet {
    type Item = &'a BoundSelection;

    type IntoIter = <&'a Vec<BoundSelection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
