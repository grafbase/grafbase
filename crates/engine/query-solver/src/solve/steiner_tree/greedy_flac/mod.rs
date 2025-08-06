use std::ops::ControlFlow;

use fixedbitset::FixedBitSet;
use petgraph::{
    data::DataMap,
    dot::{Config, Dot},
    graph::GraphIndex,
    visit::{
        Data, EdgeCount, EdgeIndexable, EdgeRef, GraphBase, IntoEdgeReferences, IntoNodeReferences, NodeCount,
        NodeIndexable,
    },
};

use crate::Cost;

mod flac;
use super::{SteinerContext, SteinerGraph};

/// Greedy FLAC-inspired Steiner tree solver that works with DAGs
/// This is a simplified version that finds shortest paths from root to terminals
pub(crate) struct GreedyFlacAlgorithm<QG: GraphBase, G: Data<EdgeWeight = Cost>> {
    ctx: SteinerContext<QG, G>,
    flac: flac::Flac,
    cost_estimerator: Option<CostEstimator<G>>,
    has_updated_cost: bool,
}

#[allow(unused)]
impl<QG: GraphBase> GreedyFlacAlgorithm<QG, SteinerGraph>
where
    QG: NodeIndexable + IntoNodeReferences + IntoEdgeReferences + EdgeCount + NodeCount + EdgeIndexable + DataMap,
    QG::NodeId: GraphIndex,
    QG::EdgeId: GraphIndex + Ord + std::fmt::Debug,
    QG::EdgeWeight: std::fmt::Debug,
    QG::NodeWeight: std::fmt::Debug,
{
    pub(crate) fn initialize(
        ctx: SteinerContext<QG, SteinerGraph>,
        terminals: impl IntoIterator<Item = QG::NodeId>,
    ) -> Self {
        let terminals = terminals
            .into_iter()
            .map(|node| ctx.to_node_ix(node))
            .collect::<Vec<_>>();

        let steiner_tree_nodes = {
            let mut nodes = FixedBitSet::with_capacity(ctx.graph.node_bound());
            // Include both the root and the root of root
            nodes.insert(ctx.root_ix.index());
            let root_of_root = ctx.graph.edge_endpoints(ctx.incoming_root_edge).unwrap().0;
            nodes.insert(root_of_root.index());
            nodes
        };

        Self {
            flac: flac::Flac::new(&ctx.graph, terminals, steiner_tree_nodes),
            cost_estimerator: None,
            ctx,
            has_updated_cost: false,
        }
    }

    pub(crate) fn to_dot_graph(
        &self,
        edge_label: impl Fn(Cost, bool) -> String,
        node_label: impl Fn(QG::NodeId, bool) -> String,
    ) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.ctx.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| {
                    let is_in_steiner_tree = self.flac.steiner_tree_nodes[edge.source().index()]
                        && self.flac.steiner_tree_nodes[edge.target().index()];
                    let cost = *edge.weight();
                    edge_label(cost, is_in_steiner_tree)
                },
                &|_, (node_ix, _)| {
                    let is_in_steiner_tree = self.flac.steiner_tree_nodes[node_ix.index()];
                    if let Some(node_id) = self.ctx.to_query_graph_node_id(node_ix) {
                        node_label(node_id, is_in_steiner_tree)
                    } else {
                        "label=\"\", style=dashed".to_string()
                    }
                }
            )
        )
    }

    pub(crate) fn insert_edge_cost_update(&mut self, _source_id: QG::NodeId, edge_id: QG::EdgeId, cost: Cost) {
        let edge_ix = self.ctx.to_edge_ix(edge_id);
        // FIXME: drop weights from the graph?
        let old = std::mem::replace(&mut self.ctx.graph[edge_ix], cost);
        self.flac.weights[edge_ix.index()] = cost;
        if let Some(estimator) = self.cost_estimerator.as_mut() {
            estimator.flac.weights[edge_ix.index()] = cost;
        }
        self.has_updated_cost |= old != cost;
    }

    pub(crate) fn extend_terminals(&mut self, extra_terminals: impl IntoIterator<Item = QG::NodeId>) {
        self.flac
            .extend_terminals(extra_terminals.into_iter().map(|node| self.ctx.to_node_ix(node)));
    }

    pub(crate) fn apply_all_cost_updates(&mut self) -> bool {
        // For this simple implementation, costs are updated immediately
        std::mem::take(&mut self.has_updated_cost)
    }

    pub(crate) fn continue_steiner_tree_growth(&mut self) -> ControlFlow<()> {
        self.flac.run(&self.ctx.graph)
    }

    pub(crate) fn estimate_extra_cost(
        &mut self,
        zero_cost_edges: &[QG::EdgeId],
        extra_terminals: &[QG::NodeId],
    ) -> Cost {
        let CostEstimator {
            mut flac,
            mut cost_backup,
        } = match self.cost_estimerator.take() {
            Some(mut estimator) => {
                estimator.flac.reset();
                estimator
                    .flac
                    .steiner_tree_nodes
                    .clone_from(&self.flac.steiner_tree_nodes);
                estimator
                    .flac
                    .steiner_tree_edges
                    .clone_from(&self.flac.steiner_tree_edges);
                estimator
            }
            None => {
                let mut flac = flac::Flac::new(&self.ctx.graph, Vec::new(), self.flac.steiner_tree_nodes.clone());
                flac.steiner_tree_edges.clone_from(&self.flac.steiner_tree_edges);
                CostEstimator {
                    flac,
                    cost_backup: Vec::new(),
                }
            }
        };

        for edge_id in zero_cost_edges {
            let edge_ix = self.ctx.to_edge_ix(*edge_id);
            let edge_cost = &mut flac.weights[edge_ix.index()];
            cost_backup.push((edge_ix, *edge_cost));
            *edge_cost = 0;
        }

        flac.extend_terminals(extra_terminals.iter().map(|node| self.ctx.to_node_ix(*node)));
        let extra_cost = flac.greedy_run(&self.ctx.graph);

        for (edge_ix, cost) in cost_backup.iter() {
            flac.weights[edge_ix.index()] = *cost;
        }

        self.cost_estimerator = Some(CostEstimator { flac, cost_backup });
        extra_cost
    }

    pub(crate) fn contains_node(&self, node_id: QG::NodeId) -> bool {
        self.flac.steiner_tree_nodes[self.ctx.to_node_ix(node_id).index()]
    }

    pub(crate) fn into_query_graph_nodes_bitset(self) -> FixedBitSet {
        let mut bitset = FixedBitSet::with_capacity(self.ctx.query_graph_node_id_to_node_ix.len());
        for (i, ix) in self.ctx.query_graph_node_id_to_node_ix.iter().copied().enumerate() {
            bitset.set(i, self.flac.steiner_tree_nodes[ix.index()]);
        }
        bitset
    }

    #[cfg(test)]
    pub(crate) fn total_cost(&self) -> Cost {
        self.flac.total_cost
    }
}

struct CostEstimator<G: GraphBase> {
    flac: flac::Flac,
    cost_backup: Vec<(G::EdgeId, Cost)>,
}
