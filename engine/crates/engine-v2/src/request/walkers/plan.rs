use engine_parser::types::OperationType;

use super::{
    BoundFieldArgumentWalker, BoundFragmentSpreadWalker, BoundInlineFragmentWalker, OperationWalker, PlanSelectionSet,
};
use crate::{
    execution::Variables,
    plan::{Attribution, AttributionWalker, PlanOutput},
};

// Not sure if best name, but those allow to walk over the "plan" for an executor. Attributed
// fields and their associated variables.
pub type PlanWalker<'a> = OperationWalker<'a, (), (), ExecutorWalkContext<'a>>;
pub type PlanOperationWalker<'a> = OperationWalker<'a, &'a PlanOutput, (), ExecutorWalkContext<'a>>;
pub type PlanFragmentSpread<'a> = BoundFragmentSpreadWalker<'a, ExecutorWalkContext<'a>>;
pub type PlanInlineFragment<'a> = BoundInlineFragmentWalker<'a, ExecutorWalkContext<'a>>;
pub type PlanFieldArgument<'a> = BoundFieldArgumentWalker<'a, ExecutorWalkContext<'a>>;

#[derive(Clone, Copy)]
pub struct ExecutorWalkContext<'a> {
    pub attribution: &'a Attribution,
    pub variables: &'a Variables<'a>,
}

impl<'a, I: Copy, SI> OperationWalker<'a, I, SI, ExecutorWalkContext<'a>> {
    pub(super) fn as_attribution_walker(&self) -> AttributionWalker<'a, I> {
        self.ctx.attribution.walk(self.item)
    }
}

impl<'a> PlanOperationWalker<'a> {
    pub fn ty(&self) -> OperationType {
        self.operation.ty
    }

    pub fn name(&self) -> Option<&'a str> {
        self.operation.name.as_deref()
    }

    pub fn selection_set(&self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::RootFields(*self)
    }
}

impl<'a> std::fmt::Debug for PlanOperationWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanOperation")
            .field("selection_set", &self.selection_set())
            .finish_non_exhaustive()
    }
}
