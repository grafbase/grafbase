use engine_parser::types::OperationType;

use super::{
    BoundFieldArgumentWalker, BoundFragmentSpreadWalker, BoundInlineFragmentWalker, OperationWalker, PlanSelectionSet,
};
use crate::{
    execution::Variables,
    plan::{Attribution, AttributionWalker, PlanOutput},
};

pub type PlanWalker<'a> = OperationWalker<'a, (), (), PlanExt<'a>>;
pub type PlanOperationWalker<'a> = OperationWalker<'a, &'a PlanOutput, (), PlanExt<'a>>;
pub type PlanFragmentSpread<'a> = BoundFragmentSpreadWalker<'a, PlanExt<'a>>;
pub type PlanInlineFragment<'a> = BoundInlineFragmentWalker<'a, PlanExt<'a>>;
pub type PlanFieldArgument<'a> = BoundFieldArgumentWalker<'a, PlanExt<'a>>;

#[derive(Clone, Copy)]
pub struct PlanExt<'a> {
    pub attribution: &'a Attribution,
    pub variables: &'a Variables<'a>,
}

impl<'a, W: Copy, S> OperationWalker<'a, W, S, PlanExt<'a>> {
    pub(super) fn as_attribution_walker(&self) -> AttributionWalker<'a, W> {
        self.ext.attribution.walk(self.wrapped)
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
