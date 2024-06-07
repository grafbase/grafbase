use super::{ExecutionError, Executor, ExecutorInput};
use crate::{execution::ExecutionContext, plan::PlanWalker, response::ResponsePart};

mod writer;

pub(crate) struct IntrospectionExecutionPlan;

impl IntrospectionExecutionPlan {
    #[allow(clippy::unnecessary_wraps)]
    pub fn new_executor<'ctx>(
        &'ctx self,
        ExecutorInput { ctx, plan, .. }: ExecutorInput<'ctx, '_>,
    ) -> Result<Executor<'ctx>, ExecutionError> {
        Ok(Executor::Introspection(IntrospectionExecutor { ctx, plan }))
    }
}

pub(crate) struct IntrospectionExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    plan: PlanWalker<'ctx>,
}

impl<'ctx> IntrospectionExecutor<'ctx> {
    pub async fn execute(self, mut response_part: ResponsePart) -> Result<ResponsePart, ExecutionError> {
        writer::IntrospectionWriter {
            schema: self.ctx.engine.schema.walker(),
            metadata: self.ctx.engine.schema.walker().introspection_metadata(),
            plan: self.plan,
            response: response_part.as_mut().next_writer().ok_or("No objects to update")?,
        }
        .execute();
        Ok(response_part)
    }
}
