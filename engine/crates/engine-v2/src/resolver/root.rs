use crate::{
    execution::{ExecutionContext, ExecutionResult},
    plan::Plan,
    response::SubgraphResponse,
    Runtime,
};

pub(super) struct RootResolver;

impl RootResolver {
    #[allow(clippy::unnecessary_wraps)]
    pub async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        Ok(subgraph_response)
    }
}
