use super::{BoundSelectionSetWalker, OperationWalker};
use crate::request::BoundInlineFragmentId;

pub type BoundInlineFragmentWalker<'a> = OperationWalker<'a, BoundInlineFragmentId>;

impl<'a> BoundInlineFragmentWalker<'a> {
    pub fn selection_set(&self) -> BoundSelectionSetWalker<'a> {
        self.walk(self.as_ref().selection_set_id)
    }
}

impl<'a> std::fmt::Debug for BoundInlineFragmentWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundInlineFragmentWalker")
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
