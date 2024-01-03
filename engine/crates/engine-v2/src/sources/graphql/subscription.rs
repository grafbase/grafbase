use schema::sources::federation::SubgraphWalker;

use super::{ExecutionContext, Executor, ExecutorError, ExecutorResult, ResolverInput};
use crate::{
    plan::PlanOutput,
    response::{ExecutorOutput, GraphqlError, ResponseBoundaryItem},
};

pub struct GraphqlSubscriptionExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    subgraph: SubgraphWalker<'ctx>,
    json_body: String,
    boundary_item: ResponseBoundaryItem,
    plan_output: PlanOutput,
    output: ExecutorOutput,
}
