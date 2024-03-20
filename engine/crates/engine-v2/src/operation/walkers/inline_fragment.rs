use super::{OperationWalker, SelectionSetWalker};
use crate::operation::InlineFragmentId;

pub type InlineFragmentWalker<'a> = OperationWalker<'a, InlineFragmentId>;

impl<'a> InlineFragmentWalker<'a> {
    pub fn selection_set(&self) -> SelectionSetWalker<'a> {
        self.walk(self.as_ref().selection_set_id)
    }
}

impl<'a> std::fmt::Debug for InlineFragmentWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InlineFragmentWalker")
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
