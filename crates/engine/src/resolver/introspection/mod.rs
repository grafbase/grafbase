use std::sync::Arc;

use crate::{
    execution::{ExecutionContext, ExecutionResult},
    operation::Plan,
    response::{InputResponseObjectSet, SubgraphResponse},
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
        input_object_refs: Arc<InputResponseObjectSet>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        let response = subgraph_response.as_shared_mut();
        for input_object_id in input_object_refs.ids() {
            writer::IntrospectionWriter {
                ctx,
                schema: ctx.schema(),
                shapes: ctx.shapes(),
                metadata: &ctx.engine.schema.subgraphs.introspection,
                plan,
                input_object_id,
                response: response.clone(),
            }
            .write(plan.shape_id());
        }
        Ok(subgraph_response)
    }
}
