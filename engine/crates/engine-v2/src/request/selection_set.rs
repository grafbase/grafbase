use engine_parser::Pos;
use schema::{Definition, FieldId, InputValueId, InterfaceId, ObjectId, UnionId};

use super::{BoundFieldDefinitionId, BoundFieldId, BoundFragmentDefinitionId, BoundSelectionSetId};
use crate::execution::StrId;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct BoundSelectionSet {
    // Ordering matters and must be respected in the response.
    pub items: Vec<BoundSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundSelection {
    Field(BoundFieldId),
    FragmentSpread(BoundFragmentSpread),
    InlineFragment(BoundInlineFragment),
}

/// The BoundFieldDefinition defines a field that is part of the actual GraphQL query.
/// A BoundField is a field in the query *after* spreading all the named fragments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundField {
    pub definition_id: BoundFieldDefinitionId,
    pub selection_set_id: BoundSelectionSetId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundFragmentSpread {
    pub location: Pos,
    pub fragment_id: BoundFragmentDefinitionId,
    // This selection set is bound to its actual position in the query.
    pub selection_set_id: BoundSelectionSetId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundInlineFragment {
    pub location: Pos,
    pub type_condition: Option<TypeCondition>,
    pub selection_set_id: BoundSelectionSetId,
    pub directives: Vec<()>,
}

/// The BoundFieldDefinition defines a field that is part of the actual GraphQL query.
/// A BoundField is a field in the query *after* spreading all the named fragments.
#[derive(Debug)]
pub struct BoundFragmentDefinition {
    pub name: String,
    pub name_location: Pos,
    pub type_condition: TypeCondition,
    pub directives: Vec<()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeCondition {
    Interface(InterfaceId),
    Object(ObjectId),
    Union(UnionId),
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
pub struct BoundFieldDefinition {
    pub name_location: Pos,
    pub name: StrId,
    pub field_id: FieldId,
    pub arguments: Vec<BoundFieldArgument>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundFieldArgument {
    pub name_location: Pos,
    pub input_value_id: InputValueId,
    pub value_location: Pos,
    // TODO: Should be validated, coerced and bound.
    pub value: engine_value::Value,
}

impl BoundSelectionSet {
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

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

impl Extend<BoundSelection> for BoundSelectionSet {
    fn extend<T: IntoIterator<Item = BoundSelection>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

impl FromIterator<BoundSelection> for BoundSelectionSet {
    fn from_iter<T: IntoIterator<Item = BoundSelection>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect::<Vec<_>>(),
        }
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
