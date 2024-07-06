mod context;
mod coordinator;
mod error;
pub(crate) mod hooks;
mod ids;
mod plan;
mod planner;
mod state;
mod walkers;

use std::sync::Arc;

use crate::{
    operation::{Operation, Variables},
    Runtime,
};
pub(crate) use context::*;
pub(crate) use coordinator::*;
pub(crate) use error::*;
pub(crate) use hooks::RequestHooks;
pub(crate) use ids::*;
pub(crate) use plan::*;
use tracing::instrument;
pub(crate) use walkers::*;

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    #[instrument(skip_all)]
    pub(crate) async fn plan_execution(
        &self,
        operation: &Operation,
        variables: &Variables,
    ) -> PlanningResult<ExecutionPlans> {
        tracing::trace!("Execution Planning");
        planner::ExecutionPlanner::new(self, operation, variables).plan().await
    }
}

/// All the necessary information for the operation to be executed that can be prepared & cached.
pub(crate) struct PreparedOperation {
    pub(crate) operation: Arc<Operation>,
    pub(crate) variables: Variables,
    pub(crate) plans: ExecutionPlans,
}

impl std::ops::Deref for PreparedOperation {
    type Target = Operation;

    fn deref(&self) -> &Self::Target {
        &self.operation
    }
}

impl<I> std::ops::Index<I> for PreparedOperation
where
    Operation: std::ops::Index<I>,
{
    type Output = <Operation as std::ops::Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.operation[index]
    }
}
