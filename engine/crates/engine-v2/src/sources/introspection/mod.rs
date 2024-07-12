use super::{ExecutionError, Executor, ExecutorInput};
use crate::{
    execution::{ExecutionContext, PlanWalker},
    response::ResponsePart,
    Runtime,
};

mod writer;

pub(crate) struct IntrospectionPreparedExecutor;

impl IntrospectionPreparedExecutor {
    #[allow(clippy::unnecessary_wraps)]
    pub fn new_executor<'ctx, R: Runtime>(
        &'ctx self,
        ExecutorInput { ctx, plan, .. }: ExecutorInput<'ctx, '_, R>,
    ) -> Result<Executor<'ctx, R>, ExecutionError> {
        Ok(Executor::Introspection(IntrospectionExecutor { ctx, plan }))
    }
}

pub(crate) struct IntrospectionExecutor<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    plan: PlanWalker<'ctx, (), ()>,
}

impl<'ctx, R: Runtime> IntrospectionExecutor<'ctx, R> {
    pub async fn execute(self, mut response_part: ResponsePart) -> Result<ResponsePart, ExecutionError> {
        writer::IntrospectionWriter {
            schema: self.ctx.engine.schema.walker(),
            metadata: self.ctx.engine.schema.walker().introspection_metadata(),
            shapes: self.plan.shapes(),
            plan: self.plan,
            response: response_part.as_mut().next_writer().ok_or("No objects to update")?,
        }
        .execute(self.plan.as_ref().output.shape_id);
        Ok(response_part)
    }
}
