use std::cell::RefCell;

use super::{ExecutionError, Executor, ExecutorInput};
use crate::{
    execution::ExecutionContext,
    plan::PlanWalker,
    response::{ResponseBoundaryItem, ResponsePart},
};

mod writer;

pub(crate) struct IntrospectionExecutionPlan;

impl IntrospectionExecutionPlan {
    #[allow(clippy::unnecessary_wraps)]
    pub fn new_executor<'ctx>(
        &'ctx self,
        ExecutorInput {
            ctx,
            boundary_objects_view: root_response_objects,
            plan,
            response_part: output,
        }: ExecutorInput<'ctx, '_>,
    ) -> Result<Executor<'ctx>, ExecutionError> {
        Ok(Executor::Introspection(IntrospectionExecutor {
            ctx,
            response_object: root_response_objects.into_single_boundary_item(),
            plan,
            output,
        }))
    }
}

pub(crate) struct IntrospectionExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    response_object: ResponseBoundaryItem,
    plan: PlanWalker<'ctx>,
    output: ResponsePart,
}

impl<'ctx> IntrospectionExecutor<'ctx> {
    pub async fn execute(mut self) -> Result<ResponsePart, ExecutionError> {
        writer::IntrospectionWriter {
            schema: self.ctx.engine.schema.walker(),
            metadata: self.ctx.engine.schema.walker().introspection_metadata(),
            plan: self.plan,
            output: RefCell::new(&mut self.output),
        }
        .update_output(self.response_object);
        Ok(self.output)
    }
}
