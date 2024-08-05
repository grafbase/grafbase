use crate::{
    execution::{ExecutionContext, ExecutionResult, PlanWalker},
    response::SubgraphResponse,
    Runtime,
};

mod writer;

pub(crate) struct IntrospectionExecutor;

impl IntrospectionExecutor {
    #[allow(clippy::unnecessary_wraps)]
    pub async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx, (), ()>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        writer::IntrospectionWriter {
            schema: ctx.engine.schema.walker(),
            metadata: ctx.engine.schema.walker().introspection_metadata(),
            shapes: &plan.blueprint().shapes,
            plan,
            response: subgraph_response.as_mut().next_writer().ok_or("No objects to update")?,
        }
        .execute(plan.logical_plan().response_blueprint().concrete_shape_id);
        Ok(subgraph_response)
    }
}
