mod model;
mod modifier;

pub(crate) use model::*;
pub(crate) use modifier::*;

use crate::{execution::PreExecutionContext, operation::Variables, Runtime};

use super::{OperationPlan, PlanResult};

impl ExecutionPlan {
    pub(super) async fn build(
        ctx: &PreExecutionContext<'_, impl Runtime>,
        operation_plan: &OperationPlan,
        variables: &Variables,
    ) -> PlanResult<ExecutionPlan> {
        let query_modifications = QueryModifications::build(ctx, operation_plan, variables).await?;
        Ok(ExecutionPlan {
            query_modifications,
            response_views: Default::default(),
            plan_resolvers: Vec::new(),
            response_modifiers: Vec::new(),
        })
    }
}
