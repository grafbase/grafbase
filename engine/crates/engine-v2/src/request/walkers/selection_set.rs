use std::collections::VecDeque;

use schema::SchemaWalker;

use super::{BoundFieldWalker, BoundFragmentSpreadWalker, BoundInlineFragmentWalker};
use crate::request::{BoundSelection, BoundSelectionSetId, Operation};

#[derive(Clone)]
pub struct BoundSelectionSetWalker<'a> {
    pub(in crate::request) schema: SchemaWalker<'a, ()>,
    pub(in crate::request) operation: &'a Operation,
    pub id: BoundSelectionSetId,
}

impl<'a> BoundSelectionSetWalker<'a> {
    pub fn fields(&self) -> FieldsIterator<'a> {
        FieldsIterator {
            selections: self.clone().into_iter(),
            nested: None,
        }
    }
}

pub enum BoundSelectionWalker<'a> {
    Field(BoundFieldWalker<'a>),
    FragmentSpread(BoundFragmentSpreadWalker<'a>),
    InlineFragment(BoundInlineFragmentWalker<'a>),
}

impl<'a> IntoIterator for BoundSelectionSetWalker<'a> {
    type Item = BoundSelectionWalker<'a>;

    type IntoIter = SelectionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SelectionIterator {
            schema: self.schema,
            operation: self.operation,
            selections: self.operation[self.id].items.iter().collect(),
        }
    }
}

pub struct SelectionIterator<'a> {
    schema: SchemaWalker<'a, ()>,
    operation: &'a Operation,
    selections: VecDeque<&'a BoundSelection>,
}

impl<'a> Iterator for SelectionIterator<'a> {
    type Item = BoundSelectionWalker<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let selection = self.selections.pop_front()?;
        Some(match selection {
            BoundSelection::Field(id) => {
                let bound_field = &self.operation[*id];
                BoundSelectionWalker::Field(BoundFieldWalker {
                    schema: self.schema,
                    operation: self.operation,
                    bound_field,
                    id: *id,
                })
            }
            BoundSelection::FragmentSpread(fragment) => {
                BoundSelectionWalker::FragmentSpread(BoundFragmentSpreadWalker {
                    schema: self.schema,
                    operation: self.operation,
                    inner: fragment,
                })
            }
            BoundSelection::InlineFragment(fragment) => {
                BoundSelectionWalker::InlineFragment(BoundInlineFragmentWalker {
                    schema: self.schema,
                    operation: self.operation,
                    inner: fragment,
                })
            }
        })
    }
}

pub struct FieldsIterator<'a> {
    selections: SelectionIterator<'a>,
    nested: Option<Box<FieldsIterator<'a>>>,
}

impl<'a> Iterator for FieldsIterator<'a> {
    type Item = BoundFieldWalker<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut nested) = self.nested {
                if let Some(field) = nested.next() {
                    return Some(field);
                }
            }
            match self.selections.next()? {
                BoundSelectionWalker::Field(field) => return Some(field),
                BoundSelectionWalker::FragmentSpread(spread) => {
                    self.nested = Some(Box::new(spread.selection_set().fields()));
                }
                BoundSelectionWalker::InlineFragment(fragment) => {
                    self.nested = Some(Box::new(fragment.selection_set().fields()));
                }
            }
        }
    }
}

impl<'a> std::fmt::Debug for BoundSelectionSetWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundSelectionSetWalker")
            .field("items", &self.clone().into_iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> std::fmt::Debug for BoundSelectionWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(field) => f.debug_tuple("Field").field(field).finish(),
            Self::FragmentSpread(spread) => f.debug_tuple("FragmentSpread").field(spread).finish(),
            Self::InlineFragment(fragment) => f.debug_tuple("InlineFragment").field(fragment).finish(),
        }
    }
}
