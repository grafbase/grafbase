mod solution;
mod solver;
mod steiner_tree;

use petgraph::stable_graph::StableGraph;
pub(crate) use solver::*;

use crate::{Edge, Node, Operation, OperationGraph};

#[allow(clippy::type_complexity)]
pub fn build_solver_with_shortest_path_algorithm<'g, 'ctx, Op: Operation>(
    operation: &'g OperationGraph<'ctx, Op>,
) -> crate::Result<
    Solver<'g, 'ctx, Op, steiner_tree::ShortestPathAlgorithm<&'g StableGraph<Node<'ctx, Op::FieldId>, Edge>>>,
> {
    Solver::initialize(operation)
}
