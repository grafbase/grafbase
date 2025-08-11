mod greedy_flac;
#[cfg(test)]
mod tests;

use fixedbitset::FixedBitSet;
pub(crate) use greedy_flac::*;
use petgraph::{
    graph::{EdgeIndex, Graph, NodeIndex},
    visit::NodeIndexable as _,
};

use crate::{Cost, solve::context::SteinerNodeId};

pub(crate) struct SteinerTree {
    pub nodes: FixedBitSet,
    pub edges: FixedBitSet,
    pub total_weight: Cost,
}

impl std::ops::Index<NodeIndex> for SteinerTree {
    type Output = bool;
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.nodes[index.index()]
    }
}

impl std::ops::Index<EdgeIndex> for SteinerTree {
    type Output = bool;
    fn index(&self, index: EdgeIndex) -> &Self::Output {
        &self.edges[index.index()]
    }
}

impl SteinerTree {
    pub fn new<N, E>(graph: &Graph<N, E>, root_node_id: SteinerNodeId) -> Self {
        let mut tree = Self {
            nodes: FixedBitSet::with_capacity(graph.node_bound()),
            edges: FixedBitSet::with_capacity(graph.edge_count()),
            total_weight: 0,
        };

        tree.nodes.insert(root_node_id.index());
        tree
    }
}
