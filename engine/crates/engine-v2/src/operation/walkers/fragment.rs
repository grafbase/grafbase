use super::{OperationWalker, SelectionSetWalker};
use crate::operation::{FragmentId, FragmentSpreadId};

pub type FragmentSpreadWalker<'a> = OperationWalker<'a, FragmentSpreadId>;

impl<'a> FragmentSpreadWalker<'a> {
    pub fn selection_set(&self) -> SelectionSetWalker<'a> {
        self.walk(self.as_ref().selection_set_id)
    }

    pub fn fragment(&self) -> FragmentDefinitionWalker<'a> {
        self.walk(self.as_ref().fragment_id)
    }
}

impl<'a> std::fmt::Debug for FragmentSpreadWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FragmentSpreadWalker")
            .field("name", &self.fragment().name())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

pub type FragmentDefinitionWalker<'a> = OperationWalker<'a, FragmentId>;

impl<'a> FragmentDefinitionWalker<'a> {
    pub fn name(&self) -> &'a str {
        &self.as_ref().name
    }
}
