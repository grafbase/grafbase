use crate::{operation::SelectionSetId, plan::AnyCollectedSelectionSetId};

use super::{PlanField, PlanWalker};

#[derive(Clone, Copy)]
pub enum PlanSelectionSet<'a> {
    RootFields(PlanWalker<'a, (), ()>),
    SelectionSet(PlanWalker<'a, SelectionSetId, ()>),
}

impl<'a> PlanSelectionSet<'a> {
    // Whether the subgraph should provide __typename (or whatever field is necessary to detect the object type)
    pub fn requires_typename(&self) -> bool {
        match self {
            PlanSelectionSet::RootFields(_) => false,
            PlanSelectionSet::SelectionSet(walker) => {
                let selection_set_id = walker.item;
                let n = usize::from(selection_set_id);
                let Some(id) = walker.operation_plan.selection_set_to_collected[n] else {
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
                    && (!collected.typename_fields.is_empty() || collected.maybe_response_object_set_id.is_some())
            }
        }
    }

    pub fn fields(&self) -> Vec<PlanField<'a>> {
        let out = match self {
            PlanSelectionSet::RootFields(walker) => walker
                .collected_selection_set()
                .fields()
                .map(move |field| field.as_operation_field())
                .collect::<Vec<_>>(),
            PlanSelectionSet::SelectionSet(walker) => {
                let plan_id = walker.operation_plan[walker.execution_plan_id].plan_id;
                walker
                    .as_ref()
                    .field_ids
                    .iter()
                    .filter_map(|id| {
                        let field_plan_id = walker.operation_plan.plan_id_for(*id);
                        if field_plan_id == plan_id {
                            walker.operation_plan[*id]
                                .definition_id()
                                .map(|definition_id| walker.walk_with(*id, definition_id))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            }
        };
        out
    }

    pub fn walker(&self) -> PlanWalker<'a, (), ()> {
        match self {
            PlanSelectionSet::RootFields(walker) => walker.walk_with((), ()),
            PlanSelectionSet::SelectionSet(walker) => walker.walk_with((), ()),
        }
    }
}

impl<'a> std::fmt::Debug for PlanSelectionSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanSelectionSet")
            .field("fields", &self.fields())
            .finish()
    }
}
