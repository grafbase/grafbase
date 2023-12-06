use std::collections::VecDeque;

use super::{BoundFieldWalker, BoundFragmentSpreadWalker, BoundInlineFragmentWalker, OperationWalker, PlanFilter};
use crate::request::{BoundSelection, BoundSelectionSetId, SelectionSetType};

pub type BoundSelectionSetWalker<'a, Extension = ()> = OperationWalker<'a, BoundSelectionSetId, (), Extension>;

pub trait SelectionSet<'a, E>
where
    Self: IntoIterator<Item = BoundSelectionWalker<'a, E>>,
{
    fn ty(&self) -> SelectionSetType;
}

impl<'a, E: PlanFilter + Copy> SelectionSet<'a, E> for BoundSelectionSetWalker<'a, E> {
    fn ty(&self) -> SelectionSetType {
        self.ty
    }
}

pub enum BoundSelectionWalker<'a, E = ()> {
    Field(BoundFieldWalker<'a, E>),
    FragmentSpread(BoundFragmentSpreadWalker<'a, E>),
    InlineFragment(BoundInlineFragmentWalker<'a, E>),
}

impl<'a, E: PlanFilter + Copy> IntoIterator for BoundSelectionSetWalker<'a, E> {
    type Item = BoundSelectionWalker<'a, E>;

    type IntoIter = SelectionIterator<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        SelectionIterator {
            walker: self.walk(()),
            selections: self.operation[self.inner].items.iter().collect(),
        }
    }
}

pub struct SelectionIterator<'a, E> {
    walker: OperationWalker<'a, (), (), E>,
    selections: VecDeque<&'a BoundSelection>,
}

impl<'a, E: PlanFilter + Copy> Iterator for SelectionIterator<'a, E> {
    type Item = BoundSelectionWalker<'a, E>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let selection = self.selections.pop_front()?;
            match selection {
                &BoundSelection::Field(id) => {
                    if self.walker.ext.field(id) {
                        return Some(BoundSelectionWalker::Field(self.walker.walk(id)));
                    }
                }
                BoundSelection::FragmentSpread(spread) => {
                    if self.walker.ext.selection_set(spread.selection_set_id) {
                        return Some(BoundSelectionWalker::FragmentSpread(self.walker.walk(spread)));
                    }
                }
                BoundSelection::InlineFragment(fragment) => {
                    if self.walker.ext.selection_set(fragment.selection_set_id) {
                        return Some(BoundSelectionWalker::InlineFragment(self.walker.walk(fragment)));
                    }
                }
            }
        }
    }
}

// pub struct FieldsIterator<'a> {
//     selections: SelectionIterator<'a>,
//     nested: Option<Box<FieldsIterator<'a>>>,
// }
//
// impl<'a> Iterator for FieldsIterator<'a> {
//     type Item = BoundFieldWalker<'a>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             if let Some(ref mut nested) = self.nested {
//                 if let Some(field) = nested.next() {
//                     return Some(field);
//                 }
//             }
//             match self.selections.next()? {
//                 BoundSelectionWalker::Field(field) => return Some(field),
//                 BoundSelectionWalker::FragmentSpread(spread) => {
//                     self.nested = Some(Box::new(spread.selection_set().fields()));
//                 }
//                 BoundSelectionWalker::InlineFragment(fragment) => {
//                     self.nested = Some(Box::new(fragment.selection_set().fields()));
//                 }
//             }
//         }
//     }
// }

impl<'a, E: PlanFilter + Copy> std::fmt::Debug for BoundSelectionSetWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundSelectionSetWalker")
            .field("items", &self.into_iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a, E: PlanFilter + Copy> std::fmt::Debug for BoundSelectionWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(field) => f.debug_tuple("Field").field(field).finish(),
            Self::FragmentSpread(spread) => f.debug_tuple("FragmentSpread").field(spread).finish(),
            Self::InlineFragment(fragment) => f.debug_tuple("InlineFragment").field(fragment).finish(),
        }
    }
}
