mod error;
mod execution;
mod solver;

use crate::{
    operation::{BoundOperation, Variables},
    prepare::{CachedOperation, PrepareContext},
    Runtime,
};
pub(crate) use error::*;
pub(crate) use execution::*;
use schema::Schema;
pub(crate) use solver::*;

pub type PlanResult<T> = Result<T, PlanError>;

#[allow(unused)]
pub fn solve(schema: &Schema, mut bound_operation: BoundOperation) -> PlanResult<OperationSolution> {
    OperationSolution::solve(schema, bound_operation)
}

#[allow(unused)]
pub async fn plan_solution(
    ctx: &mut PrepareContext<'_, impl Runtime>,
    operation: &CachedOperation,
    variables: &Variables,
) -> PlanResult<OperationPlan> {
    let query_modifications = QueryModifications::build(ctx, operation, variables).await?;
    OperationPlan::plan(ctx, operation, query_modifications)
}
