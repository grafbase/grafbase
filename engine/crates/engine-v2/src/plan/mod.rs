use schema::ResolverId;

use crate::{
    formatter::{ContextAwareDebug, FormatterContext, FormatterContextHolder},
    request::OperationPath,
    response::{ReadSelectionSet, WriteSelectionSet},
};

mod planner;
mod plans;
pub use planner::PlannedOperation;
pub use plans::{ExecutionPlans, PlanId};

pub struct ExecutionPlan {
    pub path: OperationPath,
    pub input: ReadSelectionSet,
    pub output: WriteSelectionSet,
    pub resolver_id: ResolverId,
}

impl ContextAwareDebug for ExecutionPlan {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let resolver = &ctx.schema[self.resolver_id];
        f.debug_struct("ExecutionPlan")
            .field("path", &ctx.debug(&self.path))
            .field("input", &ctx.debug(&self.input))
            .field("output", &ctx.debug(&self.output))
            .field("resolver", &resolver)
            .finish()
    }
}
