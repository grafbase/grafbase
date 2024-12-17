// #[cfg(test)]
// mod tests;

pub(crate) mod dot_graph;
mod error;
mod query;
mod solution_space;
pub(crate) mod solve;
pub use error::*;
pub use petgraph;
pub use query::*;
use schema::Schema;
pub(crate) use solution_space::*;

pub(crate) type Cost = u16;

pub fn solve(schema: &Schema, operation: &mut ::operation::Operation) -> Result<SolvedQuery> {
    let query_solution_space = Query::generate_solution_space(schema, &operation)?;
    let solution = solve::Solver::initialize(schema, &operation, &query_solution_space)?.solve()?;
    Ok(SolvedQuery::init(schema, operation, query_solution_space, solution)?.finalize())
}
