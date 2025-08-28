#![deny(clippy::future_not_send, unused_crate_dependencies)]

use grafbase_workspace_hack as _;
#[cfg(test)]
mod tests;

use grafbase_workspace_hack as _;

pub(crate) mod dot_graph;
mod error;
mod post_process;
mod query;
mod solution_space;
pub(crate) mod solve;
pub use error::*;
use operation::Operation;
pub use petgraph;
pub use query::*;
use schema::Schema;
pub(crate) use solution_space::*;

pub fn solve(schema: &Schema, operation: &mut Operation) -> Result<QuerySolution> {
    let query_solution_space = Query::generate_solution_space(schema, operation)?;
    let solution = solve::Solver::initialize(schema, operation, query_solution_space)?.solve()?;
    let crude_solved_query = solution.into_query(schema, operation)?;
    let solved_query = post_process::post_process(schema, operation, crude_solved_query);
    Ok(solved_query)
}
