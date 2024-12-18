mod error;
mod model;
mod solver;

use schema::Schema;

use super::BoundOperation;
pub(crate) use error::*;
pub(crate) use model::*;

/// Solving is divided in roughly three steps:
/// 1. Run the query_solver crate to generate the SolutionGraph, defining what resolver to use for
///    which part of query and all the field dependencies.
/// 2. Take the SolutionGraph and the BoundOperation to create all the QueryPartitions in the SolvedOperation
/// 3. Compute all the field shapes for each partition.
#[tracing::instrument(name = "solve", level = "debug", skip_all)]
pub(crate) fn solve(schema: &Schema, bound_operation: BoundOperation) -> SolveResult<SolvedOperation> {
    solver::Solver::build(schema, bound_operation)?.solve()
}
