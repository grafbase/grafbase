use super::{type_condition_name, BoundSelectionSetWalker, ExecutorWalkContext, OperationWalker, PlanSelectionSet};
use crate::request::{BoundFragmentId, BoundFragmentSpreadId};

pub type BoundFragmentSpreadWalker<'a, CtxOrUnit = ()> = OperationWalker<'a, BoundFragmentSpreadId, (), CtxOrUnit>;

impl<'a, C: Copy> BoundFragmentSpreadWalker<'a, C> {
    pub fn fragment(&self) -> BoundFragmentDefinitionWalker<'a, C> {
        self.walk_with(self.as_ref().fragment_id, ())
    }
}

impl<'a> BoundFragmentSpreadWalker<'a, ()> {
    pub fn selection_set(&self) -> BoundSelectionSetWalker<'a> {
        self.walk(self.as_ref().selection_set_id)
    }
}

impl<'a> BoundFragmentSpreadWalker<'a, ExecutorWalkContext<'a>> {
    pub fn selection_set(&self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::Query(self.walk(self.as_ref().selection_set_id))
    }
}

impl<'a> std::fmt::Debug for BoundFragmentSpreadWalker<'a, ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fragment = &self.operation[self.as_ref().fragment_id];
        f.debug_struct("BoundFragmentSpreadWalker")
            .field("name", &fragment.name)
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

impl<'a> std::fmt::Debug for BoundFragmentSpreadWalker<'a, ExecutorWalkContext<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fragment = &self.operation[self.as_ref().fragment_id];
        f.debug_struct("BoundFragmentSpreadWalker")
            .field("name", &fragment.name)
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

pub type BoundFragmentDefinitionWalker<'a, CtxOrUnit = ()> = OperationWalker<'a, BoundFragmentId, (), CtxOrUnit>;

impl<'a, C> BoundFragmentDefinitionWalker<'a, C> {
    pub fn name(&self) -> &'a str {
        &self.as_ref().name
    }

    pub fn type_condition_name(&self) -> &str {
        type_condition_name(self.schema_walker, self.as_ref().type_condition)
    }
}
