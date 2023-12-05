use super::{type_condition_name, BoundSelectionSetWalker, OperationWalker, PlanFilter};
use crate::request::{BoundFragmentDefinitionId, BoundFragmentSpread};

pub type BoundFragmentSpreadWalker<'a, Extension = ()> = OperationWalker<'a, &'a BoundFragmentSpread, (), Extension>;

impl<'a, E> std::ops::Deref for BoundFragmentSpreadWalker<'a, E> {
    type Target = BoundFragmentSpread;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, E: Copy> BoundFragmentSpreadWalker<'a, E> {
    pub fn selection_set(&self) -> BoundSelectionSetWalker<'a, E> {
        self.walk(self.inner.selection_set_id)
    }

    pub fn fragment(&self) -> BoundFragmentDefinitionWalker<'a, E> {
        self.walk_with(self.inner.fragment_id, ())
    }
}

impl<'a, E: PlanFilter + Copy> std::fmt::Debug for BoundFragmentSpreadWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fragment = &self.operation[self.inner.fragment_id];
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
        type_condition_name(self.schema, self.type_condition)
    }
}
