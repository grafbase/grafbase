use std::sync::Arc;

use crate::{
    Runtime,
    execution::{ExecutionContext, ExecutionResult},
    prepare::Plan,
    response::{ParentObjects, ResponsePart},
};

mod writer;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct IntrospectionResolver;

impl IntrospectionResolver {
    #[allow(clippy::unnecessary_wraps)]
    pub fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_object_refs: Arc<ParentObjects>,
        response: ResponsePart<'ctx>,
    ) -> ExecutionResult<ResponsePart<'ctx>> {
        let response = response.into_shared();
        for parent_object_id in parent_object_refs.ids() {
            writer::IntrospectionWriter {
                ctx,
                schema: ctx.schema(),
                shapes: ctx.shapes(),
                metadata: &ctx.schema().subgraphs.introspection,
                plan,
                parent_object_id,
                response: response.clone(),
            }
            .write(plan.shape_id());
        }
        Ok(response.unshare().unwrap())
    }
}
