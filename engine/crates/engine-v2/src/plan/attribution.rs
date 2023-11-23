use schema::Schema;

use super::PlanId;
use crate::request::{BoundFieldId, BoundSelectionSetId, BoundSelectionSetWalker, BoundSelectionWalker, Operation};

/// Keeps track of which field is assigned to which plan. It's then used to filter out all
/// irrelevant fields in a selection set for a given plan. We also track all plan ids for which a
/// selection set has fields for. This helps us avoid creating empty selection sets when iterating
/// over a planned selection set (associated with a specific plan).
pub struct Attribution {
    field_to_plan: Vec<PlanId>,
    selection_set_to_plans: Vec<Vec<PlanId>>,
}

impl std::ops::Index<BoundFieldId> for Attribution {
    type Output = PlanId;
    fn index(&self, field: BoundFieldId) -> &Self::Output {
        &self.field_to_plan[usize::from(field)]
    }
}

impl std::ops::Index<BoundSelectionSetId> for Attribution {
    type Output = Vec<PlanId>;

    fn index(&self, index: BoundSelectionSetId) -> &Self::Output {
        &self.selection_set_to_plans[usize::from(index)]
    }
}

impl Attribution {
    pub fn builder(operation: &Operation) -> AttributionBuilder {
        AttributionBuilder {
            field_to_plan: vec![None; operation.fields.len()],
        }
    }

    fn compute_plan_ids(&mut self, selection_set: BoundSelectionSetWalker<'_>) -> Vec<PlanId> {
        let id = selection_set.id;
        let plan_ids = selection_set
            .into_iter()
            .flat_map(|selection| match selection {
                BoundSelectionWalker::Field(field) => {
                    self.compute_plan_ids(field.selection_set());
                    vec![self[field.bound_field_id()]]
                }
                BoundSelectionWalker::FragmentSpread(spread) => self.compute_plan_ids(spread.selection_set()),
                BoundSelectionWalker::InlineFragment(fragment) => self.compute_plan_ids(fragment.selection_set()),
            })
            .collect::<Vec<_>>();
        self.selection_set_to_plans[usize::from(id)] = plan_ids.clone();
        plan_ids
    }
}

pub struct AttributionBuilder {
    field_to_plan: Vec<Option<PlanId>>,
}

impl AttributionBuilder {
    pub(super) fn attribute(&mut self, field: BoundFieldId, plan: PlanId) {
        self.field_to_plan[usize::from(field)] = Some(plan);
    }

    pub(super) fn build(self, schema: &Schema, operation: &Operation) -> Attribution {
        let mut attribution = Attribution {
            field_to_plan: self
                .field_to_plan
                .into_iter()
                .map(|plan| plan.expect("unattributed field"))
                .collect(),
            selection_set_to_plans: operation.selection_sets.iter().map(|_| Vec::new()).collect(),
        };
        attribution.compute_plan_ids(operation.walk_root_selection_set(schema.default_walker()));
        attribution
    }
}
