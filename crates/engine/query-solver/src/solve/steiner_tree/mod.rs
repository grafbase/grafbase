mod context;
mod greedy_flac;
#[cfg(test)]
mod tests;

pub(crate) use context::*;
use fixedbitset::FixedBitSet;
pub(crate) use greedy_flac::*;
use petgraph::{
    graph::{Graph, NodeIndex},
    visit::NodeIndexable as _,
};

use crate::Cost;

#[derive(Clone)]
pub(crate) struct SteinerTree {
    pub nodes: FixedBitSet,
    pub edges: FixedBitSet,
    pub total_weight: Cost,
}

impl SteinerTree {
    pub fn new<N, E>(graph: &Graph<N, E>, root_ix: NodeIndex) -> Self {
        let mut tree = Self {
            nodes: FixedBitSet::with_capacity(graph.node_bound()),
            edges: FixedBitSet::with_capacity(graph.edge_count()),
            total_weight: 0,
        };

        tree.nodes.insert(root_ix.index());
        tree
    }
}
