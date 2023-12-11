use super::{type_condition_name, OperationWalker, PlanExt, PlanSelectionSet};
use crate::request::BoundInlineFragment;

pub type BoundInlineFragmentWalker<'a, Extension = ()> = OperationWalker<'a, &'a BoundInlineFragment, (), Extension>;

impl<'a, E> std::ops::Deref for BoundInlineFragmentWalker<'a, E> {
    type Target = BoundInlineFragment;

    fn deref(&self) -> &Self::Target {
        self.wrapped
    }
}

impl<'a, E: Copy> BoundInlineFragmentWalker<'a, E> {
    pub fn type_condition_name(&self) -> Option<&str> {
        self.wrapped
            .type_condition
            .map(|cond| type_condition_name(self.schema_walker, cond))
    }
}

impl<'a> BoundInlineFragmentWalker<'a, PlanExt<'a>> {
    pub fn selection_set(&self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::Query(self.walk(self.wrapped.selection_set_id))
    }
}

impl<'a> std::fmt::Debug for BoundInlineFragmentWalker<'a, PlanExt<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundInlineFragmentWalker")
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
