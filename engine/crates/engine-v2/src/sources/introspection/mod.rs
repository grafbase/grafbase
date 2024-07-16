use super::{ExecutionError, Executor, ExecutorInput};
use crate::{
    execution::{ExecutionContext, ExecutionResult, PlanWalker},
    response::SubgraphResponseMutRef,
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
    pub async fn execute<'resp>(self, subgraph_response: SubgraphResponseMutRef<'resp>) -> ExecutionResult<()>
    where
        'ctx: 'resp,
    {
        writer::IntrospectionWriter {
            schema: self.ctx.engine.schema.walker(),
            metadata: self.ctx.engine.schema.walker().introspection_metadata(),
            shapes: &self.plan.blueprint().shapes,
            plan: self.plan,
            response: subgraph_response
                .into_shared()
                .next_writer()
                .ok_or("No objects to update")?,
        }
        .execute(self.plan.logical_plan().response_blueprint().concrete_shape_id);
        Ok(())
    }
}
