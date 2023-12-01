use std::num::NonZeroU16;

use super::{BoundAnyFieldDefinition, BoundField, BoundFragmentDefinition, BoundSelectionSet, Operation};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, derive_more::Display)]
pub struct BoundAnyFieldDefinitionId(u32);

impl From<usize> for BoundAnyFieldDefinitionId {
    fn from(value: usize) -> Self {
        BoundAnyFieldDefinitionId(value.try_into().expect("Too many fields."))
    }
}

impl From<BoundAnyFieldDefinitionId> for usize {
    fn from(value: BoundAnyFieldDefinitionId) -> Self {
        value.0 as usize
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, derive_more::Display)]
pub struct BoundFragmentDefinitionId(u16);

impl From<usize> for BoundFragmentDefinitionId {
    fn from(value: usize) -> Self {
        BoundFragmentDefinitionId(value.try_into().expect("Too many fragments."))
    }
}

impl From<BoundFragmentDefinitionId> for usize {
    fn from(value: BoundFragmentDefinitionId) -> Self {
        value.0 as usize
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, derive_more::Display)]
pub struct BoundSelectionSetId(NonZeroU16);

impl From<usize> for BoundSelectionSetId {
    fn from(value: usize) -> Self {
        BoundSelectionSetId(
            u16::try_from(value)
                .ok()
                .and_then(|value| NonZeroU16::new(value + 1))
                .expect("Too many selection sets."),
        )
    }
}

impl From<BoundSelectionSetId> for usize {
    fn from(value: BoundSelectionSetId) -> Self {
        (value.0.get() - 1) as usize
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, derive_more::Display)]
pub struct BoundFieldId(u32);

impl From<usize> for BoundFieldId {
    fn from(value: usize) -> Self {
        BoundFieldId(value.try_into().expect("Too many spreaded fields."))
    }
}

impl From<BoundFieldId> for usize {
    fn from(value: BoundFieldId) -> Self {
        value.0 as usize
    }
}

impl std::ops::Index<BoundFieldId> for Operation {
    type Output = BoundField;

    fn index(&self, index: BoundFieldId) -> &Self::Output {
        &self.fields[usize::from(index)]
    }
}

impl std::ops::Index<BoundSelectionSetId> for Operation {
    type Output = BoundSelectionSet;

    fn index(&self, index: BoundSelectionSetId) -> &Self::Output {
        &self.selection_sets[usize::from(index)]
    }
}

impl std::ops::Index<BoundAnyFieldDefinitionId> for Operation {
    type Output = BoundAnyFieldDefinition;

    fn index(&self, index: BoundAnyFieldDefinitionId) -> &Self::Output {
        &self.field_definitions[usize::from(index)]
    }
}

impl std::ops::Index<BoundFragmentDefinitionId> for Operation {
    type Output = BoundFragmentDefinition;

    fn index(&self, index: BoundFragmentDefinitionId) -> &Self::Output {
        &self.fragment_definitions[usize::from(index)]
    }
}
