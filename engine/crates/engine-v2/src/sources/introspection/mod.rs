use std::cell::RefCell;

use schema::sources::introspection::{Metadata, ResolverWalker};

use super::{Executor, ExecutorError, ResolverInput};
use crate::{
    execution::ExecutionContext,
    plan::{PlanId, PlanOutput},
    response::{ExecutorOutput, ResponseBoundaryItem},
};

mod writer;

pub(crate) struct IntrospectionExecutionPlan<'ctx> {
    ctx: ExecutionContext<'ctx>,
    response_object: ResponseBoundaryItem,
    metadata: &'ctx Metadata,
    plan_output: PlanOutput,
    output: ExecutorOutput,
    pub(super) plan_id: PlanId,
}

impl<'ctx> IntrospectionExecutionPlan<'ctx> {
    #[allow(clippy::unnecessary_wraps)]
    pub fn build<'input>(
        resolver: ResolverWalker<'ctx>,
        ResolverInput {
            ctx,
            boundary_objects_view: root_response_objects,
            plan_id,
            plan_output,
            output,
        }: ResolverInput<'ctx, 'input>,
    ) -> Result<Executor<'ctx>, ExecutorError> {
        Ok(Executor::Introspection(IntrospectionExecutionPlan {
            ctx,
            response_object: root_response_objects.into_single_boundary_item(),
            metadata: resolver.metadata(),
            plan_output,
            output,
            plan_id,
        }))
    }

    pub async fn execute(mut self) -> Result<ExecutorOutput, ExecutorError> {
        writer::IntrospectionWriter {
            schema: self.ctx.schema(),
            metadata: self.metadata,
            walker: self.ctx.walk(&self.plan_output),
            output: RefCell::new(&mut self.output),
        }
        .update_output(self.response_object, &self.plan_output.expectations.root_selection_set);
        Ok(self.output)
    }
}
