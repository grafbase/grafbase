use super::{type_condition_name, BoundSelectionSetWalker, OperationWalker};
use crate::request::{BoundFragmentId, BoundFragmentSpreadId};

pub type BoundFragmentSpreadWalker<'a> = OperationWalker<'a, BoundFragmentSpreadId>;

impl<'a> BoundFragmentSpreadWalker<'a> {
    pub fn selection_set(&self) -> BoundSelectionSetWalker<'a> {
        self.walk(self.as_ref().selection_set_id)
    }
}

impl<'a> std::fmt::Debug for BoundFragmentSpreadWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fragment = &self.operation[self.as_ref().fragment_id];
        f.debug_struct("BoundFragmentSpreadWalker")
            .field("name", &fragment.name)
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

pub type BoundFragmentDefinitionWalker<'a> = OperationWalker<'a, BoundFragmentId>;

impl<'a> BoundFragmentDefinitionWalker<'a> {
    pub fn name(&self) -> &'a str {
        &self.as_ref().name
    }

    pub fn type_condition_name(&self) -> &str {
        type_condition_name(self.schema_walker, self.as_ref().type_condition)
    }
}
