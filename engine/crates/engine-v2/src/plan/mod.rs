mod builder;
mod error;
mod execution;
mod model;

use crate::operation::Operation;
pub(crate) use execution::*;
pub(crate) use model::*;
use schema::Schema;

pub type PlanResult<T> = Result<T, error::PlanError>;

#[allow(unused)]
pub fn plan(schema: &Schema, mut operation: Operation) -> PlanResult<OperationPlan> {
    OperationPlan::build(schema, operation)
}
