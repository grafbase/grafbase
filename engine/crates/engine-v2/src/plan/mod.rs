mod builder;
mod error;
mod execution;
mod model;

use crate::{
    execution::PreExecutionContext,
    operation::{Operation, Variables},
    Runtime,
};
pub(crate) use execution::*;
pub(crate) use model::*;
use schema::Schema;

pub type PlanResult<T> = Result<T, error::PlanError>;

#[allow(unused)]
pub fn plan(schema: &Schema, mut operation: Operation) -> PlanResult<OperationPlan> {
    OperationPlan::build(schema, operation)
}

#[allow(unused)]
pub async fn create_execution_plan(
    ctx: &PreExecutionContext<'_, impl Runtime>,
    operation_plan: &OperationPlan,
    variables: &Variables,
) -> PlanResult<ExecutionPlan> {
    let query_modifications = QueryModifications::build(ctx, operation_plan, variables).await?;
    Ok(todo!())
}
