use super::{type_condition_name, BoundSelectionSetWalker, OperationWalker, PlanFilter};
use crate::request::BoundInlineFragment;

pub type BoundInlineFragmentWalker<'a, Extension = ()> = OperationWalker<'a, &'a BoundInlineFragment, (), Extension>;

impl<'a, E> std::ops::Deref for BoundInlineFragmentWalker<'a, E> {
    type Target = BoundInlineFragment;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, E: Copy> BoundInlineFragmentWalker<'a, E> {
    pub fn type_condition_name(&self) -> Option<&str> {
        self.inner
            .type_condition
            .map(|cond| type_condition_name(self.schema, cond))
    }

    pub fn selection_set(&self) -> BoundSelectionSetWalker<'a, E> {
        self.walk(self.inner.selection_set_id)
    }
}

impl<'a, E: PlanFilter + Copy> std::fmt::Debug for BoundInlineFragmentWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundInlineFragmentWalker")
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
