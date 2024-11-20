mod graph;
mod shortest_path;
#[cfg(test)]
mod tests;

use fixedbitset::FixedBitSet;
use graph::*;
use petgraph::visit::{GraphBase, IntoEdgeReferences, IntoNodeReferences};
pub(crate) use shortest_path::*;

use crate::Cost;

pub(crate) trait SteinerTreeAlgorithm<G: GraphBase + IntoNodeReferences + IntoEdgeReferences> {
    fn initialize(
        operation_graph: G,
        node_filter: impl Fn(G::NodeRef) -> Option<G::NodeId>,
        edge_filter: impl Fn(G::EdgeRef) -> Option<(G::EdgeId, G::NodeId, G::NodeId, Cost)>,
        root: G::NodeId,
        terminals: impl IntoIterator<Item = G::NodeId>,
    ) -> Self;

    /// Core function that actually moves forward the algorithm. At each step we include at least
    /// one new terminal with a non-zero cost.
    /// The control is given back to the caller allowing for any edge cost updates as the cost of requirements might have changed.
    /// The return value indicates whether we still have any missing terminals left.
    fn continue_steiner_tree_growth(&mut self) -> bool;

    // Whether a node is part of the Steiner tree.
    fn contains_node(&self, node_id: G::NodeId) -> bool;

    // Estimate the extra cost of retrieving additional terminals with the current Steiner tree and
    // a few edges force set to zero cost.
    //
    // Used to estimate the cost of all requirements of a resolver when taking a certain path.
    fn estimate_extra_cost(
        &mut self,
        zero_cost_edges: impl IntoIterator<Item = G::EdgeId>,
        extra_terminals: impl IntoIterator<Item = G::NodeId>,
    ) -> Cost;

    /// Pushes an edge cost update to the algorithm. This will be applied before the next growth phase at the latest.
    /// We don't apply them immediately to avoid re-computing the shortest paths all the time.
    fn insert_edge_cost_update(&mut self, source_id: G::NodeId, edge_id: G::EdgeId, cost: Cost);

    /// Forces all cost updates to be applied.
    fn apply_all_cost_updates(&mut self) -> bool;

    /// Add new terminals that must be reached. Typically those will be requirements based on the
    /// resolvers that were chosen.
    fn extend_terminals(&mut self, extra_terminals: impl IntoIterator<Item = G::NodeId>);

    /// Bitset indicating whether nodes in the operation graph are part of the SteinerTree.
    fn operation_graph_bitset(&self) -> FixedBitSet;

    /// Represent the current state of the Steiner Tree as a dot graph
    fn to_dot_graph(
        &self,
        edge_label: impl Fn(Cost, bool) -> String,
        node_label: impl Fn(G::NodeId, bool) -> String,
    ) -> String;
}
