use std::collections::VecDeque;

use schema::{Definition, DefinitionWalker};

use super::{BoundFieldWalker, BoundFragmentSpreadWalker, BoundInlineFragmentWalker, OperationWalker};
use crate::request::{BoundSelection, BoundSelectionSetId, SelectionSetType};

pub type BoundSelectionSetWalker<'a, Extension = ()> = OperationWalker<'a, BoundSelectionSetId, (), Extension>;
pub type SelectionSetTypeWalker<'a, Extension = ()> = OperationWalker<'a, SelectionSetType, Definition, Extension>;

impl<'a, E> BoundSelectionSetWalker<'a, E> {
    pub fn ty(&self) -> SelectionSetTypeWalker<'a, ()> {
        self.with_ext(()).walk_with(self.ty, Definition::from(self.ty))
    }
}

impl<'a, E> std::ops::Deref for SelectionSetTypeWalker<'a, E> {
    type Target = DefinitionWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

pub enum BoundSelectionWalker<'a, E = ()> {
    Field(BoundFieldWalker<'a, E>),
    FragmentSpread(BoundFragmentSpreadWalker<'a, E>),
    InlineFragment(BoundInlineFragmentWalker<'a, E>),
}

impl<'a, E: Copy> IntoIterator for BoundSelectionSetWalker<'a, E> {
    type Item = BoundSelectionWalker<'a, E>;

    type IntoIter = SelectionIterator<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        SelectionIterator {
            walker: self.walk(()),
            selections: self.operation[self.wrapped].items.iter().collect(),
        }
    }
}

pub struct SelectionIterator<'a, E> {
    walker: OperationWalker<'a, (), (), E>,
    selections: VecDeque<&'a BoundSelection>,
}

impl<'a, E: Copy> Iterator for SelectionIterator<'a, E> {
    type Item = BoundSelectionWalker<'a, E>;

    fn next(&mut self) -> Option<Self::Item> {
        let selection = self.selections.pop_front()?;
        Some(match selection {
            &BoundSelection::Field(id) => BoundSelectionWalker::Field(self.walker.walk(id)),
            BoundSelection::FragmentSpread(spread) => BoundSelectionWalker::FragmentSpread(self.walker.walk(spread)),
            BoundSelection::InlineFragment(fragment) => {
                BoundSelectionWalker::InlineFragment(self.walker.walk(fragment))
            }
        })
    }
}

impl<'a, E: Copy> std::fmt::Debug for BoundSelectionSetWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundSelectionSet")
            .field("ty", &self.ty().name())
            .field("items", &self.into_iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a, E> std::fmt::Debug for BoundSelectionWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(field) => field.fmt(f),
            Self::FragmentSpread(spread) => spread.fmt(f),
            Self::InlineFragment(fragment) => fragment.fmt(f),
        }
    }
}
