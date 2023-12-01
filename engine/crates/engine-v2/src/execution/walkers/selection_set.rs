use std::collections::VecDeque;

use engine_parser::Pos;

use super::{FieldWalker, FragmentSpreadWalker, InlineFragmentWalker};
use crate::request::{BoundAnyFieldDefinition, BoundSelection, BoundSelectionSetId, SelectionSetRoot};

#[derive(Clone)]
pub struct SelectionSetWalker<'a> {
    pub(super) ctx: super::WalkerContext<'a, ()>,
    pub(super) merged_selection_set_ids: Vec<BoundSelectionSetId>,
}

impl<'a> SelectionSetWalker<'a> {
    pub fn root(&self) -> SelectionSetRoot {
        self.ctx.operation[self.merged_selection_set_ids[0]].root
    }
}

pub enum SelectionWalker<'a> {
    Field(FieldWalker<'a>),
    FragmentSpread(FragmentSpreadWalker<'a>),
    InlineFragment(InlineFragmentWalker<'a>),
}

impl<'a> SelectionWalker<'a> {
    #[allow(dead_code)]
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
            selections: self
                .merged_selection_set_ids
                .iter()
                .flat_map(|id| self.ctx.operation[*id].items.iter())
                .collect(),
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
                    if self.ctx.attribution.field(*id) {
                        let bound_field = self.ctx.operation[*id];
                        match &self.ctx.operation[bound_field.definition_id] {
                            BoundAnyFieldDefinition::TypeName(_) => continue,
                            BoundAnyFieldDefinition::Field(definition) => {
                                return Some(SelectionWalker::Field(super::FieldWalker {
                                    ctx: self.ctx.walk(definition.field_id),
                                    definition,
                                    bound_field,
                                }))
                            }
                        }
                    }
                }
                BoundSelection::FragmentSpread(spread) => {
                    if self.ctx.attribution.selection_set(spread.selection_set_id) {
                        return Some(SelectionWalker::FragmentSpread(FragmentSpreadWalker {
                            ctx: self.ctx,
                            inner: spread,
                        }));
                    }
                }
                BoundSelection::InlineFragment(fragment) => {
                    if self.ctx.attribution.selection_set(fragment.selection_set_id) {
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
