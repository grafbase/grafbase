use std::collections::VecDeque;

use schema::SchemaWalker;

use super::{BoundFieldWalker, BoundFragmentSpreadWalker, BoundInlineFragmentWalker};
use crate::request::{path::ResolvedTypeCondition, BoundSelection, BoundSelectionSetId, Operation};

#[derive(Clone)]
pub struct BoundSelectionSetWalker<'a> {
    pub(in crate::request) schema: SchemaWalker<'a, ()>,
    pub(in crate::request) operation: &'a Operation,
    pub id: BoundSelectionSetId,
}

impl<'a> BoundSelectionSetWalker<'a> {
    // Flatten all fields irrelevant of fragments. Only useful when type conditions are irrelevant.
    pub fn flatten_fields(&self) -> FlattenFieldsIterator<'a> {
        FlattenFieldsIterator {
            resolved_type_condition: None,
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
                    schema_field: self.schema.walk(self.operation[bound_field.definition_id].field_id),
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

pub struct FlattenFieldsIterator<'a> {
    pub(super) resolved_type_condition: Option<ResolvedTypeCondition>,
    pub(super) selections: SelectionIterator<'a>,
    pub(super) nested: Option<Box<FlattenFieldsIterator<'a>>>,
}

#[derive(Debug)]
pub struct FlattenedBoundField<'a> {
    pub resolved_type_condition: Option<ResolvedTypeCondition>,
    pub inner: BoundFieldWalker<'a>,
}

impl<'a> std::ops::Deref for FlattenedBoundField<'a> {
    type Target = BoundFieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> Iterator for FlattenFieldsIterator<'a> {
    type Item = FlattenedBoundField<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut nested) = self.nested {
            if let Some(item) = nested.next() {
                return Some(item);
            }
        }
        let selection = self.selections.next()?;
        match selection {
            BoundSelectionWalker::Field(inner) => Some(FlattenedBoundField {
                resolved_type_condition: self.resolved_type_condition.clone(),
                inner,
            }),
            BoundSelectionWalker::FragmentSpread(fragment) => {
                self.nested = Some(Box::new(fragment.nested_fields(self.resolved_type_condition.clone())));
                self.next()
            }
            BoundSelectionWalker::InlineFragment(fragment) => {
                self.nested = Some(Box::new(fragment.nested_fields(self.resolved_type_condition.clone())));
                self.next()
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
