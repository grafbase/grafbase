mod greedy_flac;
#[cfg(test)]
mod tests;

use std::ops::ControlFlow;

use fixedbitset::FixedBitSet;
pub(crate) use greedy_flac::*;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    visit::EdgeIndexable as _,
};

use crate::solve::input::{SteinerNodeId, SteinerWeight};

pub(crate) struct SteinerTree {
    pub nodes: FixedBitSet,
    pub edges: FixedBitSet,
    pub total_weight: SteinerWeight,
    pub terminals: Vec<SteinerNodeId>,
    pub is_terminal: FixedBitSet,
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
    pub fn new<N, E>(
        graph: &petgraph::Graph<N, E>,
        root_node_id: SteinerNodeId,
        terminals: Vec<SteinerNodeId>,
    ) -> Self {
        use petgraph::visit::NodeIndexable as _;

        let mut tree = Self {
            nodes: FixedBitSet::with_capacity(graph.node_bound()),
            edges: FixedBitSet::with_capacity(graph.edge_bound()),
            total_weight: 0,
            terminals,
            is_terminal: FixedBitSet::with_capacity(graph.node_bound()),
        };

        for t in tree.terminals.iter() {
            tree.is_terminal.insert(t.index());
        }

        tree.nodes.insert(root_node_id.index());
        tree
    }

    pub fn extend_terminals(&mut self, new_terminals: impl IntoIterator<Item = SteinerNodeId>) -> ControlFlow<()> {
        let n = self.terminals.len();
        self.terminals.extend(
            new_terminals
                .into_iter()
                .filter(|node_id| !self.is_terminal.put(node_id.index()) && !self.nodes[node_id.index()]),
        );
        if n != self.terminals.len() {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }

    pub fn clone_from_with_new_terminals(
        &mut self,
        other: &Self,
        terminals: impl IntoIterator<Item = SteinerNodeId>,
    ) -> ControlFlow<()> {
        self.nodes.clone_from(&other.nodes);
        self.edges.clone_from(&other.edges);
        self.total_weight = 0;
        self.terminals.clear();
        self.is_terminal.clear();
        self.extend_terminals(terminals)
    }
}
