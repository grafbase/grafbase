use std::cell::RefCell;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    response::{ParentObjectSet, ResponsePartBuilder},
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
        parent_objects: ParentObjectSet,
        response: ResponsePartBuilder<'ctx>,
    ) -> ResponsePartBuilder<'ctx> {
        let response = RefCell::new(response);
        let writer = writer::IntrospectionWriter {
            ctx,
            schema: ctx.schema(),
            shapes: ctx.shapes(),
            metadata: &ctx.schema().subgraphs.introspection,
            plan,
            response,
        };
        for parent_object in parent_objects.iter() {
            writer.write(parent_object, plan.shape());
        }
        writer.response.into_inner()
    }
}
