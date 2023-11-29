use std::collections::VecDeque;

use engine_parser::Pos;
use schema::ObjectId;

use super::{FieldWalker, FragmentSpreadWalker, GroupedFieldSet, InlineFragmentWalker};
use crate::request::{BoundSelection, BoundSelectionSetId};

#[derive(Clone)]
pub struct SelectionSetWalker<'a> {
    pub(super) ctx: super::WalkerContext<'a, ()>,
    pub(super) id: BoundSelectionSetId,
}

impl<'a> SelectionSetWalker<'a> {
    pub fn collect_fields(&self, concrete_object_id: ObjectId) -> GroupedFieldSet<'a> {
        let mut grouped_field_set = GroupedFieldSet::new(self.ctx, concrete_object_id);
        grouped_field_set.collect_fields(self.id);
        grouped_field_set
    }
}

pub enum SelectionWalker<'a> {
    Field(FieldWalker<'a>),
    FragmentSpread(FragmentSpreadWalker<'a>),
    InlineFragment(InlineFragmentWalker<'a>),
}

impl<'a> SelectionWalker<'a> {
    pub fn location(&self) -> Pos {
        match self {
            SelectionWalker::Field(field) => field.location(),
            SelectionWalker::FragmentSpread(spread) => spread.location(),
            SelectionWalker::InlineFragment(fragment) => fragment.location(),
        }
    }
}

impl<'a> IntoIterator for SelectionSetWalker<'a> {
    type Item = SelectionWalker<'a>;

    type IntoIter = PlannedSelectionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PlannedSelectionIterator {
            ctx: self.ctx,
            selections: self.ctx.plan.operation[self.id].items.iter().collect(),
        }
    }
}

pub struct PlannedSelectionIterator<'a> {
    ctx: super::WalkerContext<'a, ()>,
    selections: VecDeque<&'a BoundSelection>,
}

impl<'a> Iterator for PlannedSelectionIterator<'a> {
    type Item = SelectionWalker<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(selection) = self.selections.pop_front() {
            match selection {
                BoundSelection::Field(id) => {
                    if self.ctx.plan.attribution[*id] == self.ctx.plan_id {
                        let bound_field = self.ctx.plan.operation[*id];
                        return Some(SelectionWalker::Field(super::FieldWalker {
                            ctx: self
                                .ctx
                                .walk(self.ctx.plan.operation[bound_field.definition_id].field_id),
                            bound_field,
                        }));
                    }
                }
                BoundSelection::FragmentSpread(spread) => {
                    if self.ctx.plan.attribution[spread.selection_set_id].contains(&self.ctx.plan_id) {
                        return Some(SelectionWalker::FragmentSpread(FragmentSpreadWalker {
                            ctx: self.ctx,
                            inner: spread,
                        }));
                    }
                }
                BoundSelection::InlineFragment(fragment) => {
                    if self.ctx.plan.attribution[fragment.selection_set_id].contains(&self.ctx.plan_id) {
                        return Some(SelectionWalker::InlineFragment(InlineFragmentWalker {
                            ctx: self.ctx,
                            inner: fragment,
                        }));
                    }
                }
            }
        }
        None
    }
}

impl<'a> std::fmt::Debug for SelectionSetWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionSetWalker")
            .field("items", &self.clone().into_iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> std::fmt::Debug for SelectionWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(field) => f.debug_tuple("Field").field(field).finish(),
            Self::FragmentSpread(spread) => f.debug_tuple("FragmentSpread").field(spread).finish(),
            Self::InlineFragment(fragment) => f.debug_tuple("InlineFragment").field(fragment).finish(),
        }
    }
}
