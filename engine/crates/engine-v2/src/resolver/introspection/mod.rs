use crate::{
    execution::{ExecutionContext, ExecutionResult},
    plan::Plan,
    response::SubgraphResponse,
    Runtime,
};

mod writer;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct IntrospectionResolver;

impl IntrospectionResolver {
    #[allow(clippy::unnecessary_wraps)]
    pub async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        writer::IntrospectionWriter {
            ctx,
            schema: ctx.schema(),
            shapes: ctx.shapes(),
            metadata: &ctx.engine.schema.subgraphs.introspection,
            plan,
            response: subgraph_response.as_mut().next_writer().ok_or("No objects to update")?,
        }
        .execute(plan.shape_id());
        Ok(subgraph_response)
    }
}
