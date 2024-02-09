use crate::request::{BoundFragmentDefinitionWalker, BoundFragmentSpreadId};

use super::{PlanSelectionSet, PlanWalker};

pub type PlanFragmentSpread<'a> = PlanWalker<'a, BoundFragmentSpreadId, ()>;

impl<'a> PlanFragmentSpread<'a> {
    pub fn selection_set(&self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::SelectionSet(self.walk(self.as_ref().selection_set_id))
    }

    pub fn fragment(&self) -> BoundFragmentDefinitionWalker<'a> {
        self.bound_walk_with(self.as_ref().fragment_id, ())
    }
}

impl<'a> std::fmt::Debug for PlanFragmentSpread<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fragment = &self.operation_plan[self.as_ref().fragment_id];
        f.debug_struct("PlanFragmentSpread")
            .field("name", &fragment.name)
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
