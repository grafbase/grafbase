use std::collections::VecDeque;

use schema::Definition;

use super::{
    BoundFragmentSpreadWalker, BoundInlineFragmentWalker, BoundSelectionSetWalker, ExecutorWalkContext,
    OperationWalker, PlanField, PlanOperationWalker, PlanWalker, SelectionSetTypeWalker,
};
use crate::{
    plan::{ExtraFieldWalker, ExtraSelectionSetId},
    request::{BoundFieldId, BoundSelection, SelectionSetType},
};

#[derive(Clone)]
pub enum PlanSelectionSet<'a> {
    RootFields(PlanOperationWalker<'a>),
    Query(BoundSelectionSetWalker<'a, ExecutorWalkContext<'a>>),
    Extra(OperationWalker<'a, ExtraSelectionSetId, (), ExecutorWalkContext<'a>>),
}

impl<'a> PlanSelectionSet<'a> {
    pub fn ty(&self) -> SelectionSetTypeWalker<'a, ()> {
        match self {
            Self::RootFields(walker) => {
                let ty: SelectionSetType = walker.item.entity_type.into();
                walker.walk_with(ty, Definition::from(ty)).without_ctx()
            }
            Self::Query(walker) => walker.ty(),
            Self::Extra(walker) => {
                let ty = walker.as_attribution_walker().ty();
                walker.walk_with(ty, Definition::from(ty)).without_ctx()
            }
        }
    }
}

pub enum PlanSelection<'a> {
    Field(PlanField<'a>),
    FragmentSpread(BoundFragmentSpreadWalker<'a, ExecutorWalkContext<'a>>),
    InlineFragment(BoundInlineFragmentWalker<'a, ExecutorWalkContext<'a>>),
}

impl<'a> IntoIterator for PlanSelectionSet<'a> {
    type Item = PlanSelection<'a>;

    type IntoIter = PlanSelectionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::RootFields(walker) => PlanSelectionIterator {
                walker: walker.walk(()),
                bound_field_ids: walker.item.root_fields.iter().copied().collect(),
                selections: VecDeque::with_capacity(0),
                extra_fields: walker
                    .ctx
                    .attribution
                    .extras_for(walker.item.root_selection_set_id)
                    .map(|extras| extras.fields().collect())
                    .unwrap_or_default(),
            },
            Self::Query(walker) => PlanSelectionIterator {
                walker: walker.walk(()),
                bound_field_ids: VecDeque::with_capacity(0),
                selections: walker.operation[walker.item].items.iter().collect(),
                extra_fields: walker
                    .ctx
                    .attribution
                    .extras_for(walker.item)
                    .map(|extras| extras.fields().collect())
                    .unwrap_or_default(),
            },
            Self::Extra(walker) => PlanSelectionIterator {
                walker: walker.walk(()),
                bound_field_ids: VecDeque::with_capacity(0),
                selections: VecDeque::with_capacity(0),
                extra_fields: walker.as_attribution_walker().fields().collect(),
            },
        }
    }
}

pub struct PlanSelectionIterator<'a> {
    walker: PlanWalker<'a>,
    bound_field_ids: VecDeque<BoundFieldId>,
    selections: VecDeque<&'a BoundSelection>,
    extra_fields: VecDeque<ExtraFieldWalker<'a>>,
}

impl<'a> Iterator for PlanSelectionIterator<'a> {
    type Item = PlanSelection<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(id) = self.bound_field_ids.pop_front() {
                let bound_field = self.walker.walk(id);
                // Skipping over metadata fields. The plan doesn't provide them.
                let field = bound_field.schema_field().map(|schema_field| {
                    PlanSelection::Field(PlanField::Query(
                        bound_field.walk_with(bound_field.id(), schema_field.id()),
                    ))
                });
                if field.is_some() {
                    return field;
                }
            } else if let Some(selection) = self.selections.pop_front() {
                match selection {
                    &BoundSelection::Field(id) => {
                        if self.walker.ctx.attribution.field(id) {
                            self.bound_field_ids.push_back(id);
                        }
                    }
                    BoundSelection::FragmentSpread(id) => {
                        let spread = self.walker.walk(*id);
                        if self
                            .walker
                            .ctx
                            .attribution
                            .selection_set(spread.as_ref().selection_set_id)
                        {
                            return Some(PlanSelection::FragmentSpread(spread));
                        }
                    }
                    BoundSelection::InlineFragment(id) => {
                        let inline_fragment = self.walker.walk(*id);
                        if self
                            .walker
                            .ctx
                            .attribution
                            .selection_set(inline_fragment.as_ref().selection_set_id)
                        {
                            return Some(PlanSelection::InlineFragment(inline_fragment));
                        }
                    }
                }
            } else {
                return self.extra_fields.pop_front().map(|extra_field| {
                    PlanSelection::Field(PlanField::Extra(
                        self.walker.walk_with(extra_field.id(), extra_field.field_id),
                    ))
                });
            }
        }
    }
}

impl<'a> std::fmt::Debug for PlanSelectionSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanSelectionSet")
            .field("ty", &self.ty().name())
            .field("items", &self.clone().into_iter().collect::<Vec<_>>())
            .finish()
    }
}

impl<'a> std::fmt::Debug for PlanSelection<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field(field) => field.fmt(f),
            Self::FragmentSpread(spread) => spread.fmt(f),
            Self::InlineFragment(fragment) => fragment.fmt(f),
        }
    }
}
