use std::collections::VecDeque;

use schema::{Definition, DefinitionWalker};

use super::{
    BoundFragmentSpreadWalker, BoundInlineFragmentWalker, OperationWalker, PlanExt, PlanField, PlanOperationWalker,
    PlanWalker,
};
use crate::{
    plan::{ExtraFieldWalker, ExtraSelectionSetId},
    request::{BoundFieldId, BoundSelection, BoundSelectionSetId, SelectionSetType},
};

pub type BoundSelectionSetWalker<'a, Extension = ()> = OperationWalker<'a, BoundSelectionSetId, (), Extension>;
pub type SelectionSetTypeWalker<'a, Extension = ()> = OperationWalker<'a, SelectionSetType, Definition, Extension>;

impl<'a, E> std::ops::Deref for SelectionSetTypeWalker<'a, E> {
    type Target = DefinitionWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

#[derive(Clone)]
pub enum PlanSelectionSet<'a> {
    RootFields(PlanOperationWalker<'a>),
    Query(BoundSelectionSetWalker<'a, PlanExt<'a>>),
    Extra(OperationWalker<'a, ExtraSelectionSetId, (), PlanExt<'a>>),
}

impl<'a> PlanSelectionSet<'a> {
    pub fn ty(&self) -> SelectionSetTypeWalker<'a, ()> {
        match self {
            Self::RootFields(walker) => {
                let ty: SelectionSetType = walker.wrapped.entity_type.into();
                walker.walk_with(ty, Definition::from(ty)).with_plan(())
            }
            Self::Query(walker) => walker.walk_with(walker.ty, Definition::from(walker.ty)).with_plan(()),
            Self::Extra(walker) => {
                let ty = walker.as_attribution_walker().ty();
                walker.walk_with(ty, Definition::from(ty)).with_plan(())
            }
        }
    }
}

pub enum PlanSelection<'a> {
    Field(PlanField<'a>),
    FragmentSpread(BoundFragmentSpreadWalker<'a, PlanExt<'a>>),
    InlineFragment(BoundInlineFragmentWalker<'a, PlanExt<'a>>),
}

impl<'a> IntoIterator for PlanSelectionSet<'a> {
    type Item = PlanSelection<'a>;

    type IntoIter = PlanSelectionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::RootFields(walker) => PlanSelectionIterator {
                walker: walker.walk(()),
                bound_field_ids: walker.wrapped.root_fields.iter().copied().collect(),
                selections: VecDeque::with_capacity(0),
                extra_fields: VecDeque::with_capacity(0),
            },
            Self::Query(walker) => PlanSelectionIterator {
                walker: walker.walk(()),
                bound_field_ids: VecDeque::with_capacity(0),
                selections: walker.operation[walker.wrapped].items.iter().collect(),
                extra_fields: walker
                    .ext
                    .attribution
                    .extras_for(walker.wrapped)
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
                let field = bound_field.definition().as_field().map(|definition| {
                    PlanSelection::Field(PlanField::Query(
                        bound_field.walk_with((bound_field.id(), definition.wrapped), definition.id()),
                    ))
                });
                if field.is_some() {
                    return field;
                }
            } else if let Some(selection) = self.selections.pop_front() {
                match selection {
                    &BoundSelection::Field(id) => {
                        if self.walker.ext.attribution.field(id) {
                            self.bound_field_ids.push_back(id);
                        }
                    }
                    BoundSelection::FragmentSpread(spread) => {
                        if self.walker.ext.attribution.selection_set(spread.selection_set_id) {
                            return Some(PlanSelection::FragmentSpread(self.walker.walk(spread)));
                        }
                    }
                    BoundSelection::InlineFragment(fragment) => {
                        if self.walker.ext.attribution.selection_set(fragment.selection_set_id) {
                            return Some(PlanSelection::InlineFragment(self.walker.walk(fragment)));
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

impl<'a, E> std::fmt::Debug for BoundSelectionSetWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundSelectionSetWalker").finish_non_exhaustive()
    }
}

impl<'a> std::fmt::Debug for PlanSelectionSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanSelectionSet")
            .field("ty", &self.ty().name())
            .field("fields", &self.clone().into_iter().collect::<Vec<_>>())
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
