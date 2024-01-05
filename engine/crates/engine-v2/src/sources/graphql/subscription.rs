use schema::sources::federation::SubgraphWalker;

use super::ExecutionContext;
use crate::{
    plan::PlanOutput,
    response::{ExecutorOutput, ResponseBoundaryItem},
};

#[allow(dead_code)]
pub struct GraphqlSubscriptionExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    subgraph: SubgraphWalker<'ctx>,
    json_body: String,
    boundary_item: ResponseBoundaryItem,
    plan_output: PlanOutput,
    output: ExecutorOutput,
}
