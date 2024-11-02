mod adapter;
mod builder;
mod error;
mod model;

use crate::operation::Operation;
use adapter::OperationAdapter;
pub(crate) use model::*;
use schema::Schema;

pub type PlanResult<T> = Result<T, error::PlanError>;

#[allow(unused)]
pub fn plan(schema: &Schema, mut operation: Operation) -> PlanResult<OperationPlan> {
    let graph = query_planning::OperationGraph::new(schema, OperationAdapter::new(schema, &mut operation))?.solve()?;
    OperationPlan::build(schema, operation, graph)
}
