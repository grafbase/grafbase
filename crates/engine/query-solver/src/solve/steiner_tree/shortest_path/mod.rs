#![allow(unused)]

mod tree;

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
use tree::ShortestPathSteinerTree;

use crate::Cost;

use super::{SteinerContext, SteinerGraph};

/// Very simple Steiner tree solver. We compute the shortest path from the root to the terminals
/// and add those incrementally to the Steiner Tree.
///
/// Contrary to a "standard" Steiner Tree algorithm we need and assume a few things:
/// - The "Steiner graph" is a DAG
/// - Cost of edges can change while building the Steiner Tree.
pub(crate) struct ShortestPathAlgorithm<QG: GraphBase, G: Data<EdgeWeight = Cost>> {
    ctx: SteinerContext<QG, G>,
    missing_terminals: Vec<G::NodeId>,
    cost_update: CostUpdate<G>,
    steiner_tree: ShortestPathSteinerTree<G>,
    tmp: Reusable<G>,
}

/// Temporary structures re-used across SteinerTree algorithm invocation
struct Reusable<G: GraphBase> {
    lowered_cost_nodes: Vec<G::NodeId>,
    cost_backup: Vec<(G::EdgeId, Cost)>,
    terminals_buffer: Vec<G::NodeId>,
    steiner_tree_copy: ShortestPathSteinerTree<G>,
}

/// Keeps track of the cost updates to apply to the graph.
#[derive(Default, Debug)]
struct CostUpdate<G: GraphBase> {
    increases_cost: bool,
    records: Vec<(G::NodeId, G::EdgeId, Cost)>,
}

impl<QG: GraphBase> ShortestPathAlgorithm<QG, SteinerGraph>
where
    QG: NodeIndexable + IntoNodeReferences + IntoEdgeReferences + EdgeCount + NodeCount + EdgeIndexable + DataMap,
    QG::NodeId: GraphIndex,
    QG::EdgeId: GraphIndex + Ord + std::fmt::Debug,
    QG::EdgeWeight: std::fmt::Debug,
    QG::NodeWeight: std::fmt::Debug,
{
    #[allow(unused)]
    pub(crate) fn initialize(
        ctx: SteinerContext<QG, SteinerGraph>,
        terminals: impl IntoIterator<Item = QG::NodeId>,
    ) -> Self {
        let steiner_tree = ShortestPathSteinerTree::build(&ctx);
        let missing_terminals = terminals
            .into_iter()
            .map(|node| ctx.to_node_ix(node))
            .collect::<Vec<_>>();

        let mut alg = Self {
            ctx,
            missing_terminals,
            cost_update: CostUpdate::default(),
            tmp: Reusable {
                lowered_cost_nodes: Vec::new(),
                cost_backup: Vec::new(),
                terminals_buffer: Vec::new(),
                steiner_tree_copy: steiner_tree.clone(),
            },
            steiner_tree,
        };

        // Initialize the shortest paths, currently everything is set to MAX. We should have
        // reached all terminals at this point with a sensible cost.
        alg.regenerate_shortest_paths();
        // debug_assert!(alg
        //     .missing_terminals
        //     .iter()
        //     .all(|t| alg.steiner_tree.shortest_paths[t.index()].cumulative_cost_from_root < Cost::MAX));

        alg
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
                    let is_in_steiner_tree =
                        self.steiner_tree.contains(edge.source()) && self.steiner_tree.contains(edge.target());
                    let cost = *edge.weight();
                    edge_label(cost, is_in_steiner_tree)
                },
                &|_, (node_ix, _)| {
                    let is_in_steiner_tree = self.steiner_tree.contains(node_ix);
                    if let Some(node_id) = self.ctx.to_query_graph_node_id(node_ix) {
                        node_label(node_id, is_in_steiner_tree)
                    } else {
                        debug_assert!(!is_in_steiner_tree);
                        "label=\"\", style=dashed".to_string()
                    }
                }
            )
        )
    }

    /// Pushes an edge cost update to the algorithm. This will be applied before the next growth phase at the latest.
    /// We don't apply them immediately to avoid re-computing the shortest paths all the time.
    pub(crate) fn insert_edge_cost_update(&mut self, source_id: QG::NodeId, edge_id: QG::EdgeId, cost: Cost) {
        let edge_ix = self.ctx.to_edge_ix(edge_id);
        match self.ctx.graph[edge_ix].cmp(&cost) {
            std::cmp::Ordering::Equal => (),
            std::cmp::Ordering::Less => {
                self.cost_update.increases_cost = true;
                self.cost_update
                    .records
                    .push((self.ctx.to_node_ix(source_id), edge_ix, cost));
            }
            std::cmp::Ordering::Greater => {
                self.cost_update
                    .records
                    .push((self.ctx.to_node_ix(source_id), edge_ix, cost));
            }
        }
    }

    /// Add new terminals that must be reached. Typically those will be requirements based on the
    /// resolvers that were chosen.
    pub(crate) fn extend_terminals(&mut self, extra_terminals: impl IntoIterator<Item = QG::NodeId>) {
        self.missing_terminals
            .extend(extra_terminals.into_iter().map(|node| self.ctx.to_node_ix(node)));
    }

    /// Forces all cost updates to be applied.
    pub(crate) fn apply_all_cost_updates(&mut self) -> bool {
        if !self.cost_update.records.is_empty() {
            if self.cost_update.increases_cost {
                for (_, edge, cost) in self.cost_update.records.drain(..) {
                    self.ctx.graph[edge] = cost;
                }
                self.regenerate_shortest_paths();
            } else {
                debug_assert!(self.tmp.lowered_cost_nodes.is_empty());
                for (source, edge, cost) in self.cost_update.records.drain(..) {
                    self.ctx.graph[edge] = cost;
                    self.tmp.lowered_cost_nodes.push(source);
                }
                self.steiner_tree
                    .update_shortest_paths(&self.ctx, &mut self.tmp.lowered_cost_nodes);
            }
            true
        } else {
            false
        }
    }

    fn regenerate_shortest_paths(&mut self) {
        self.steiner_tree.reset_shortest_paths(&self.ctx);

        debug_assert!(self.tmp.lowered_cost_nodes.is_empty());
        self.tmp.lowered_cost_nodes.push(self.ctx.root_ix);
        self.steiner_tree
            .update_shortest_paths(&self.ctx, &mut self.tmp.lowered_cost_nodes);
    }

    /// Core function that actually moves forward the algorithm. At each step we include at least
    /// one new terminal with a non-zero cost.
    /// The control is given back to the caller allowing for any edge cost updates as the cost of requirements might have changed.
    /// The return value indicates whether we still have any missing terminals left.
    pub(crate) fn continue_steiner_tree_growth(&mut self) -> ControlFlow<()> {
        // Ensure we start from a clean state.
        self.apply_all_cost_updates();

        let mut all_zero_cost_and_first_non_zero_cost_terminals = {
            self.missing_terminals
                .sort_unstable_by_key(|a| self.steiner_tree.node_addition_cost(*a));

            let partition_point = self
                .missing_terminals
                .partition_point(|t| self.steiner_tree.node_addition_cost(*t) == 0);

            let end = (partition_point + 1).min(self.missing_terminals.len());
            let mut terminals = self.missing_terminals.split_off(end);
            std::mem::swap(&mut terminals, &mut self.missing_terminals);
            terminals
        };

        self.steiner_tree.grow_with_some_terminals(
            &mut self.ctx,
            &mut all_zero_cost_and_first_non_zero_cost_terminals,
            &mut self.tmp,
        );

        // We added at least one non-zero edge to the Steiner tree. Shortest paths are
        // automatically updated when growing. So we might have other terminals now that are
        // zero-cost. We process them immediately as there is no need to re-calibrate requirement
        // cost for them.
        if !self.missing_terminals.is_empty() {
            let mut all_zero_cost_terminals = {
                self.missing_terminals
                    .sort_unstable_by_key(|a| self.steiner_tree.node_addition_cost(*a));

                let partition_point = self
                    .missing_terminals
                    .partition_point(|t| self.steiner_tree.node_addition_cost(*t) == 0);

                let end = partition_point.min(self.missing_terminals.len());
                let mut all_zero_cost_terminals = self.missing_terminals.split_off(end);
                std::mem::swap(&mut all_zero_cost_terminals, &mut self.missing_terminals);
                all_zero_cost_terminals
            };

            self.steiner_tree
                .grow_with_some_terminals(&mut self.ctx, &mut all_zero_cost_terminals, &mut self.tmp);
        }

        if self.missing_terminals.is_empty() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    // Estimate the extra cost of retrieving additional terminals with the current Steiner tree and
    // a few edges force set to zero cost.
    //
    // Used to estimate the cost of all requirements of a resolver when taking a certain path.
    pub(crate) fn estimate_extra_cost(
        &mut self,
        zero_cost_edges: &[QG::EdgeId],
        extra_terminals: &[QG::NodeId],
    ) -> Cost {
        debug_assert!(self.tmp.terminals_buffer.is_empty());
        self.tmp.terminals_buffer.extend(
            extra_terminals
                .iter()
                .map(|node| self.ctx.to_node_ix(*node))
                .filter(|node_ix| !self.steiner_tree.contains(*node_ix)),
        );

        if self
            .tmp
            .terminals_buffer
            .iter()
            .all(|terminal_ix| self.steiner_tree.node_addition_cost(*terminal_ix) == 0)
        {
            self.tmp.terminals_buffer.clear();
            return 0;
        } else if let [terminal_ix] = self.tmp.terminals_buffer[..] {
            if zero_cost_edges.is_empty() {
                self.tmp.terminals_buffer.clear();
                return self.steiner_tree.node_addition_cost(terminal_ix);
            }
        }

        debug_assert!(self.tmp.lowered_cost_nodes.is_empty() && self.tmp.cost_backup.is_empty());
        for edge_id in zero_cost_edges {
            let edge_ix = self.ctx.to_edge_ix(*edge_id);
            let edge_cost = &mut self.ctx.graph[edge_ix];
            if *edge_cost > 0 {
                self.tmp.cost_backup.push((edge_ix, *edge_cost));
                *edge_cost = 0;
                let source_ix = self.ctx.graph.edge_endpoints(edge_ix).unwrap().0;
                self.tmp.lowered_cost_nodes.push(source_ix);
            }
        }

        self.tmp.steiner_tree_copy.clone_from(&self.steiner_tree);
        self.tmp
            .steiner_tree_copy
            .update_shortest_paths(&self.ctx, &mut self.tmp.lowered_cost_nodes);
        self.tmp
            .terminals_buffer
            .sort_by_key(|t| self.tmp.steiner_tree_copy.node_addition_cost(*t));
        self.tmp
            .steiner_tree_copy
            .grow_with_all_missing_terminals(&self.ctx, &mut self.tmp.terminals_buffer);

        self.tmp.terminals_buffer.clear();
        for (edge_ix, cost) in self.tmp.cost_backup.drain(..) {
            self.ctx.graph[edge_ix] = cost;
        }

        self.tmp.steiner_tree_copy.cost - self.steiner_tree.cost
    }

    // Whether a node is part of the Steiner tree.
    pub(crate) fn contains_node(&self, node_id: QG::NodeId) -> bool {
        self.steiner_tree.contains(self.ctx.to_node_ix(node_id))
    }

    pub(crate) fn into_query_graph_nodes_bitset(self) -> FixedBitSet {
        let mut bitset = FixedBitSet::with_capacity(self.ctx.query_graph_node_id_to_node_ix.len());
        for (i, ix) in self.ctx.query_graph_node_id_to_node_ix.iter().copied().enumerate() {
            bitset.set(i, self.steiner_tree.contains(ix));
        }
        bitset
    }

    #[cfg(test)]
    pub(crate) fn total_cost(&self) -> Cost {
        self.steiner_tree.cost
    }
}
