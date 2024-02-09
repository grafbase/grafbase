use schema::Definition;

use crate::{
    plan::AnyCollectedSelectionSetId,
    request::{BoundSelection, BoundSelectionSetId, SelectionSetTypeWalker},
};

use super::{PlanField, PlanFragmentSpread, PlanInlineFragment, PlanWalker};

#[derive(Clone, Copy)]
pub enum PlanSelectionSet<'a> {
    RootFields(PlanWalker<'a, (), ()>),
    SelectionSet(PlanWalker<'a, BoundSelectionSetId, ()>),
}

impl<'a> PlanSelectionSet<'a> {
    pub fn ty(&self) -> SelectionSetTypeWalker<'a> {
        match self {
            PlanSelectionSet::RootFields(walker) => {
                let id = walker.operation_plan.plan_outputs[usize::from(walker.plan_id)].collected_selection_set_id;
                let ty = walker.operation_plan[id].ty;
                walker.bound_walk_with(ty, Definition::from(ty))
            }
            PlanSelectionSet::SelectionSet(selection_set) => {
                let ty = selection_set.as_ref().ty;
                selection_set.bound_walk_with(ty, Definition::from(ty))
            }
        }
    }

    // Whether the subgraph should provide __typename (or whatever field is necessary to detect the object type)
    pub fn requires_typename(&self) -> bool {
        match self {
            PlanSelectionSet::RootFields(_) => false,
            PlanSelectionSet::SelectionSet(walker) => {
                let selection_set_id = walker.item;
                let n = usize::from(selection_set_id);
                let Some(id) = walker.operation_plan.bound_to_collected_selection_set[n] else {
                    // Means we're not a root selection set, meaning we're flattened inside another
                    // one. We're a inline fragment / fragment spread.
                    return false;
                };
                let AnyCollectedSelectionSetId::Collected(id) = id else {
                    // If we're not "concrete", it means there are type conditions we couldn't
                    // resolve during planning and thus need the __typename
                    return true;
                };
                // If we couldn't determine the object_id during planning and we have __typename
                // fields, we need to have it
                let collected = &walker.operation_plan[id];
                collected.ty.as_object_id().is_none()
                    && (!collected.typename_fields.is_empty() || collected.maybe_boundary_id.is_some())
            }
        }
    }
}

impl<'a> IntoIterator for PlanSelectionSet<'a> {
    type Item = PlanSelection<'a>;
    type IntoIter = PlanSelectionSetIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        PlanSelectionSetIterator {
            selection_set: self,
            next_index: 0,
        }
    }
}

pub enum PlanSelection<'a> {
    Field(PlanField<'a>),
    FragmentSpread(PlanFragmentSpread<'a>),
    InlineFragment(PlanInlineFragment<'a>),
}

pub struct PlanSelectionSetIterator<'a> {
    selection_set: PlanSelectionSet<'a>,
    next_index: usize,
}

impl<'a> Iterator for PlanSelectionSetIterator<'a> {
    type Item = PlanSelection<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.selection_set {
            PlanSelectionSet::RootFields(plan) => {
                let id = plan.collected_selection_set().as_ref().fields.get(self.next_index)?;
                self.next_index += 1;
                let field = &plan.operation_plan[id];
                return Some(PlanSelection::Field(
                    plan.walk_with(field.bound_field_id, field.schema_field_id),
                ));
            }
            PlanSelectionSet::SelectionSet(selection_set) => loop {
                let selection = selection_set.as_ref().items.get(self.next_index)?;
                self.next_index += 1;
                let plan_id = selection_set.plan_id;
                let operation = selection_set.operation_plan;
                return Some(match selection {
                    BoundSelection::Field(id) => {
                        let Some(field_id) = operation[*id].schema_field_id() else {
                            continue;
                        };
                        if operation.bound_field_to_plan_id[usize::from(*id)] != plan_id {
                            continue;
                        }
                        PlanSelection::Field(selection_set.walk_with(*id, field_id))
                    }
                    BoundSelection::FragmentSpread(id) => {
                        let spread = &operation[*id];
                        if operation.bound_selection_to_plan_id[usize::from(spread.selection_set_id)] != plan_id {
                            continue;
                        }
                        PlanSelection::FragmentSpread(selection_set.walk(*id))
                    }
                    BoundSelection::InlineFragment(id) => {
                        let fragment = &operation[*id];
                        if operation.bound_selection_to_plan_id[usize::from(fragment.selection_set_id)] != plan_id {
                            continue;
                        }
                        PlanSelection::InlineFragment(selection_set.walk(*id))
                    }
                });
            },
        }
    }
}

impl<'a> std::fmt::Debug for PlanSelectionSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = (*self).into_iter().collect::<Vec<_>>();
        f.debug_struct("PlanSelectionSet")
            .field("ty", &self.ty().name())
            .field("items", &items)
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
