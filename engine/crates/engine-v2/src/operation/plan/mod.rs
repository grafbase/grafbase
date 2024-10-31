use adapter::BoundOperationAdapter;
use schema::Schema;

use super::Operation;

mod adapter;
mod error;

pub type PlanResult<T> = Result<T, error::PlanError>;

pub fn plan(schema: &Schema, mut operation: Operation) -> PlanResult<()> {
    let mut graph = query_planning::OperationGraph::new(
        schema,
        BoundOperationAdapter {
            schema,
            operation: &mut operation,
        },
    )?;
    let mut solver = graph.solver()?;
    solver.solve()?;
    Ok(())
}
