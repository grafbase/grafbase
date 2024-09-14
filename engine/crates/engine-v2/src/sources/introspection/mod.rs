use crate::{
    execution::{ExecutionContext, ExecutionResult},
    operation::PlanWalker,
    response::SubgraphResponse,
    Runtime,
};

mod writer;

pub(crate) struct IntrospectionResolver;

impl IntrospectionResolver {
    /// Executes the introspection query and returns the updated subgraph response.
    ///
    /// # Parameters
    ///
    /// - `ctx`: The execution context containing the necessary runtime information and schema.
    /// - `plan`: The plan walker that contains the blueprint of the logical plan.
    /// - `subgraph_response`: The current response object which will be updated with introspection data.
    ///
    /// # Returns
    ///
    /// Returns an `ExecutionResult` containing the updated `SubgraphResponse`.
    ///
    /// # Errors
    ///
    /// This function will return an error if there are no objects to update in the response.
    #[allow(clippy::unnecessary_wraps)]
    pub async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx, ()>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        writer::IntrospectionWriter {
            schema: &ctx.engine.schema,
            metadata: &ctx.engine.schema.subgraphs.introspection,
            shapes: &plan.blueprint().shapes,
            plan,
            response: subgraph_response.as_mut().next_writer().ok_or("No objects to update")?,
        }
        .execute(plan.logical_plan().response_blueprint().concrete_shape_id);

        Ok(subgraph_response)
    }
}
