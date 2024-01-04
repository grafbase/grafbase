use super::{type_condition_name, BoundSelectionSetWalker, ExecutorWalkContext, OperationWalker, PlanSelectionSet};
use crate::request::BoundInlineFragment;

pub type BoundInlineFragmentWalker<'a, CtxOrUnit = ()> = OperationWalker<'a, &'a BoundInlineFragment, (), CtxOrUnit>;

impl<'a, C: Copy> BoundInlineFragmentWalker<'a, C> {
    pub fn type_condition_name(&self) -> Option<&str> {
        self.item
            .type_condition
            .map(|cond| type_condition_name(self.schema_walker, cond))
    }
}

impl<'a> BoundInlineFragmentWalker<'a, ()> {
    pub fn selection_set(&self) -> BoundSelectionSetWalker<'a> {
        self.walk(self.item.selection_set_id)
    }
}

impl<'a> BoundInlineFragmentWalker<'a, ExecutorWalkContext<'a>> {
    pub fn selection_set(&self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::Query(self.walk(self.item.selection_set_id))
    }
}

impl<'a> std::fmt::Debug for BoundInlineFragmentWalker<'a, ExecutorWalkContext<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundInlineFragmentWalker")
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

impl<'a> std::fmt::Debug for BoundInlineFragmentWalker<'a, ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundInlineFragmentWalker")
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
