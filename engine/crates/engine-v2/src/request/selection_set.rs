use std::borrow::Cow;

use schema::{Definition, FieldId, InputValueId, InterfaceId, ObjectId, Schema, UnionId};

use crate::response::{BoundResponseKey, ResponseKey};

use super::{BoundAnyFieldDefinitionId, BoundFieldId, BoundFragmentDefinitionId, BoundSelectionSetId, Location};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundSelectionSet {
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

impl SelectionSetType {
    pub fn maybe_from(definition: Definition) -> Option<Self> {
        match definition {
            Definition::Object(id) => Some(SelectionSetType::Object(id)),
            Definition::Interface(id) => Some(Self::Interface(id)),
            Definition::Union(id) => Some(Self::Union(id)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundSelection {
    Field(BoundFieldId),
    FragmentSpread(BoundFragmentSpread),
    InlineFragment(BoundInlineFragment),
}

/// The BoundFieldDefinition defines a field that is part of the actual GraphQL query.
/// A BoundField is a field in the query *after* spreading all the named fragments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoundField {
    pub bound_response_key: BoundResponseKey,
    pub definition_id: BoundAnyFieldDefinitionId,
    pub selection_set_id: Option<BoundSelectionSetId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundFragmentSpread {
    pub location: Location,
    pub fragment_id: BoundFragmentDefinitionId,
    // This selection set is bound to its actual position in the query.
    pub selection_set_id: BoundSelectionSetId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundInlineFragment {
    pub location: Location,
    pub type_condition: Option<TypeCondition>,
    pub selection_set_id: BoundSelectionSetId,
    pub directives: Vec<()>,
}

#[derive(Debug)]
pub struct BoundFragmentDefinition {
    pub name: String,
    pub name_location: Location,
    pub type_condition: TypeCondition,
    pub directives: Vec<()>,
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

/// The BoundFieldDefinition defines a field that is part of the actual GraphQL query.
/// A BoundField is a field in the query *after* spreading all the named fragments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundAnyFieldDefinition {
    TypeName(BoundTypeNameFieldDefinition),
    Field(BoundFieldDefinition),
}

impl BoundAnyFieldDefinition {
    pub fn as_field(&self) -> Option<&BoundFieldDefinition> {
        match self {
            BoundAnyFieldDefinition::TypeName(_) => None,
            BoundAnyFieldDefinition::Field(field) => Some(field),
        }
    }

    pub fn response_key(&self) -> ResponseKey {
        match self {
            BoundAnyFieldDefinition::TypeName(field) => field.response_key,
            BoundAnyFieldDefinition::Field(field) => field.response_key,
        }
    }

    pub fn name_location(&self) -> Location {
        match self {
            BoundAnyFieldDefinition::TypeName(field) => field.name_location,
            BoundAnyFieldDefinition::Field(field) => field.name_location,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundTypeNameFieldDefinition {
    pub name_location: Location,
    pub response_key: ResponseKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundFieldDefinition {
    pub name_location: Location,
    pub response_key: ResponseKey,
    pub field_id: FieldId,
    pub arguments: Vec<BoundFieldArgument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundFieldArgument {
    pub name_location: Location,
    pub input_value_id: InputValueId,
    pub value_location: Location,
    // TODO: Should be validated, coerced and bound.
    pub value: engine_value::Value,
}

impl BoundSelectionSet {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &BoundSelection> {
        self.items.iter()
    }
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
