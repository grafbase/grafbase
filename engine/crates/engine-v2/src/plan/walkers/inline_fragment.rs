use crate::request::{type_condition_name, BoundInlineFragmentId};

use super::{PlanSelectionSet, PlanWalker};

pub type PlanInlineFragment<'a> = PlanWalker<'a, BoundInlineFragmentId, ()>;

impl<'a> PlanInlineFragment<'a> {
    pub fn selection_set(&self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::SelectionSet(self.walk(self.as_ref().selection_set_id))
    }

    pub fn type_condition_name(&self) -> Option<&str> {
        self.as_ref()
            .type_condition
            .map(|cond| type_condition_name(self.schema_walker, cond))
    }
}

impl<'a> std::fmt::Debug for PlanInlineFragment<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanInlineFragment")
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
