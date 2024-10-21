use crate::{
    execution::{ExecutionContext, ExecutionResult},
    operation::PlanWalker,
    response::SubgraphResponse,
    Runtime,
};

mod writer;

pub(crate) struct IntrospectionResolver;

impl IntrospectionResolver {
    #[allow(clippy::unnecessary_wraps)]
    pub async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx, ()>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        crate::utils::block_in_place(|| {
            writer::IntrospectionWriter {
                schema: &ctx.engine.schema,
                metadata: &ctx.engine.schema.subgraphs.introspection,
                shapes: &plan.blueprint().shapes,
                plan,
                response: subgraph_response.as_mut().next_writer().ok_or("No objects to update")?,
            }
            .execute(plan.logical_plan().response_blueprint().concrete_shape_id);
            ExecutionResult::Ok(subgraph_response)
        })
    }
}
