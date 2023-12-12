use super::{type_condition_name, OperationWalker, PlanExt, PlanSelectionSet};
use crate::request::{BoundFragmentDefinitionId, BoundFragmentSpread};

pub type BoundFragmentSpreadWalker<'a, Extension = ()> = OperationWalker<'a, &'a BoundFragmentSpread, (), Extension>;

impl<'a, E> std::ops::Deref for BoundFragmentSpreadWalker<'a, E> {
    type Target = BoundFragmentSpread;

    fn deref(&self) -> &Self::Target {
        self.wrapped
    }
}

impl<'a, E: Copy> BoundFragmentSpreadWalker<'a, E> {
    pub fn fragment(&self) -> BoundFragmentDefinitionWalker<'a, E> {
        self.walk_with(self.wrapped.fragment_id, ())
    }
}

impl<'a> BoundFragmentSpreadWalker<'a, PlanExt<'a>> {
    pub fn selection_set(&self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::Query(self.walk(self.wrapped.selection_set_id))
    }
}

impl<'a> std::fmt::Debug for BoundFragmentSpreadWalker<'a, PlanExt<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fragment = &self.operation[self.wrapped.fragment_id];
        f.debug_struct("BoundFragmentSpreadWalker")
            .field("name", &fragment.name)
            .field("selection_set", &self.selection_set())
            .finish()
    }
}

pub type BoundFragmentDefinitionWalker<'a, Extension = ()> =
    OperationWalker<'a, BoundFragmentDefinitionId, (), Extension>;

impl<'a, E> BoundFragmentDefinitionWalker<'a, E> {
    pub fn type_condition_name(&self) -> &str {
        type_condition_name(self.schema_walker, self.type_condition)
    }
}
