mod shortest_path;
#[cfg(test)]
mod tests;

use bitvec::vec::BitVec;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    stable_graph::StableGraph,
};

use crate::Cost;

#[allow(unused)]
pub(crate) struct Solution {
    pub terminals: Vec<NodeIndex>,
    pub steiner_tree_nodes: BitVec,
    pub total_cost: Cost,
}

pub(crate) type ResolverGraph<N, E> = StableGraph<N, E>;

/// Steiner tree algorithm.
///
/// The goal is to find the minimum cost tree that connects a set of terminals to the root.
///
/// See https://en.wikipedia.org/wiki/Steiner_tree_problem
#[allow(unused)]
pub(crate) trait SteinerTreeAlg<'a, N, E> {
    /// Graph is expected to be a DAG. Root should not be part of the terminals.
    fn init(graph: &'a ResolverGraph<N, E>, root: NodeIndex, terminals: Vec<NodeIndex>) -> Self;

    /// All algorithms construct the tree incrementally. It's expected to at least connect one terminal
    /// to the root, but may add more. Exposing the inner loop allows the caller to modify the cost
    /// and terminals during this process. We need to do both for operations because of
    /// requirements. Depending on the chosen edges cost will change and we may have to add new terminals (required
    /// nodes)
    fn grow_steiner_tree<F>(&mut self, edge_cost: F) -> Option<Solution>
    where
        F: Fn(EdgeIndex, &E) -> Cost;

    /// BitSet indicating whether a node is part of the Steiner Tree.
    fn steiner_tree_nodes(&self) -> &BitVec;

    /// Mutable list of missing terminals
    fn missing_terminals(&mut self) -> &mut Vec<NodeIndex>;
}
