use schema::ResolverId;

use crate::{
    formatter::{ContextAwareDebug, FormatterContext, FormatterContextHolder},
    request::{OperationPath, OperationSelectionSet},
    response::ReadSelectionSet,
};

mod planner;
mod plans;
mod tracker;

pub use planner::{OperationPlan, PrepareError};
pub use plans::{ExecutionPlans, PlanId};
pub use tracker::ExecutableTracker;

pub struct ExecutionPlan {
    pub path: OperationPath,
    pub input: ReadSelectionSet,
    pub selection_set: OperationSelectionSet,
    pub resolver_id: ResolverId,
}

impl ContextAwareDebug for ExecutionPlan {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let resolver = &ctx.schema[self.resolver_id];
        f.debug_struct("ExecutionPlan")
            .field("path", &ctx.debug(&self.path))
            .field("input", &ctx.debug(&self.input))
            .field("selection_set", &ctx.debug(&self.selection_set))
            .field("resolver", &resolver)
            .finish()
    }
}
