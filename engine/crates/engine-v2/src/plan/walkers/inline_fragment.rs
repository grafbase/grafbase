use super::{PlanSelectionSet, PlanWalker};
use crate::operation::InlineFragmentId;

pub type PlanInlineFragment<'a> = PlanWalker<'a, InlineFragmentId, ()>;

impl<'a> PlanInlineFragment<'a> {
    pub fn selection_set(&self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::SelectionSet(self.walk(self.as_ref().selection_set_id))
    }
}

impl<'a> std::fmt::Debug for PlanInlineFragment<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanInlineFragment")
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
