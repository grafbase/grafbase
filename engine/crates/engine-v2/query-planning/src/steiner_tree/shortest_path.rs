use petgraph::{
    data::DataMap,
    dot::{Config, Dot},
    graph::{EdgeIndex, NodeIndex},
    visit::{
        EdgeCount, EdgeIndexable, EdgeRef, GraphBase, IntoEdgeReferences, IntoNodeReferences, NodeCount, NodeIndexable,
    },
};

use crate::Cost;

use super::SteinerGraph;

/// Very simple Steiner tree solver. We compute the shortest path from the root to the terminals
/// and add those incrementally to the Steiner Tree.
///
/// Contrary to a "standard" Steiner Tree algorithm we need and assume a few things:
/// - The "Steiner graph" is a DAG
/// - Cost of edges can change while building the Steiner Tree.
pub(crate) struct ShortestPathAlgorithm<G: GraphBase> {
    steiner_graph: SteinerGraph<G>,
    root_ix: NodeIndex,
    missing_terminals: Vec<NodeIndex>,
    cost_update: CostUpdate,
    steiner_tree: ShortestPathSteinerTree,
    tmp: Reusable,
}

/// Temporary structures re-used across SteinerTree algorithm invocation
struct Reusable {
    lowered_cost_nodes: Vec<NodeIndex>,
    cost_backup: Vec<(EdgeIndex, Cost)>,
    terminals_buffer: Vec<NodeIndex>,
    steiner_tree_copy: ShortestPathSteinerTree,
}

/// Keeps track of the cost updates to apply to the graph.
#[derive(Default, Debug)]
struct CostUpdate {
    increases_cost: bool,
    records: Vec<(NodeIndex, EdgeIndex, Cost)>,
}

impl<G: GraphBase> ShortestPathAlgorithm<G>
where
    G: NodeIndexable + IntoNodeReferences + IntoEdgeReferences + EdgeCount + NodeCount + EdgeIndexable + DataMap,
    G::EdgeId: Ord + std::fmt::Debug,
    G::EdgeWeight: std::fmt::Debug,
    G::NodeWeight: std::fmt::Debug,
{
    pub(crate) fn initialize(
        operation_graph: G,
        node_filter: impl Fn(G::NodeRef) -> Option<G::NodeId>,
        edge_filter: impl Fn(G::EdgeRef) -> Option<(G::EdgeId, G::NodeId, G::NodeId, Cost)>,
        root: G::NodeId,
        terminals: impl IntoIterator<Item = G::NodeId>,
    ) -> Self {
        let mut steiner_graph = SteinerGraph::build(operation_graph, node_filter, edge_filter);
        let root_ix = steiner_graph.to_node_ix(root);
        let missing_terminals = terminals
            .into_iter()
            .map(|node| steiner_graph.to_node_ix(node))
            .collect::<Vec<_>>();

        let steiner_tree = ShortestPathSteinerTree {
            nodes: {
                let mut nodes = fixedbitset::FixedBitSet::with_capacity(steiner_graph.graph.node_count());
                nodes.put(root_ix.index());
                nodes
            },
            shortest_paths: {
                let dummy_root = steiner_graph.graph.add_node(());
                let dummy_incoming_root_edge_ix = steiner_graph.graph.add_edge(dummy_root, root_ix, 0);
                let mut shortest_paths = vec![
                    ShortestPath {
                        parent: root_ix,
                        incoming_edge_ix: dummy_incoming_root_edge_ix,
                        cumulative_cost_from_root: Cost::MAX
                    };
                    steiner_graph.graph.node_count()
                ];
                shortest_paths[root_ix.index()] = ShortestPath {
                    parent: root_ix,
                    incoming_edge_ix: dummy_incoming_root_edge_ix,
                    cumulative_cost_from_root: 0,
                };
                shortest_paths
            },
            cost: 0,
        };

        let mut alg = Self {
            steiner_graph,
            root_ix,
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
        debug_assert!(alg
            .missing_terminals
            .iter()
            .all(|t| alg.steiner_tree.shortest_paths[t.index()].cumulative_cost_from_root < Cost::MAX));

        alg
    }

    pub(crate) fn to_dot_graph(
        &self,
        edge_label: impl Fn(Cost, bool) -> String,
        node_label: impl Fn(G::NodeId, bool) -> String,
    ) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.steiner_graph.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| {
                    let is_in_steiner_tree = self.steiner_tree.nodes[edge.source().index()]
                        && self.steiner_tree.nodes[edge.target().index()];
                    let cost = *edge.weight();
                    edge_label(cost, is_in_steiner_tree)
                },
                &|_, (node_ix, _)| {
                    let is_in_steiner_tree = self.steiner_tree.nodes[node_ix.index()];
                    if let Some(node_id) = self.steiner_graph.to_operation_graph_node_id(node_ix) {
                        node_label(node_id, is_in_steiner_tree)
                    } else {
                        debug_assert!(!is_in_steiner_tree);
                        "label=\"\", style=dashed".to_string()
                    }
                }
            )
        )
    }

    fn debug_dot_graph(&self) -> String {
        self.to_dot_graph(
            |cost, steiner| match (cost > 0, steiner) {
                (true, true) => format!("label = \"{cost}\""),
                (true, false) => format!("label = \"{cost}\", style=dashed"),
                (false, true) => String::new(),
                (false, false) => "style=dashed".to_string(),
            },
            |node, steiner| {
                if steiner {
                    format!(
                        "label=\"{}\"",
                        self.steiner_tree
                            .node_addition_cost(self.steiner_graph.to_node_ix(node))
                    )
                } else {
                    format!(
                        "label=\"{}\", style=dashed",
                        self.steiner_tree
                            .node_addition_cost(self.steiner_graph.to_node_ix(node))
                    )
                }
            },
        )
    }

    /// Pushes an edge cost update to the algorithm. This will be applied before the next growth phase at the latest.
    /// We don't apply them immediately to avoid re-computing the shortest paths all the time.
    pub(crate) fn insert_edge_cost_update(&mut self, source_id: G::NodeId, edge_id: G::EdgeId, cost: Cost) {
        let edge_ix = self.steiner_graph.to_edge_ix(edge_id);
        match self.steiner_graph.graph[edge_ix].cmp(&cost) {
            std::cmp::Ordering::Equal => (),
            std::cmp::Ordering::Less => {
                self.cost_update.increases_cost = true;
                self.cost_update
                    .records
                    .push((self.steiner_graph.to_node_ix(source_id), edge_ix, cost));
            }
            std::cmp::Ordering::Greater => {
                self.cost_update
                    .records
                    .push((self.steiner_graph.to_node_ix(source_id), edge_ix, cost));
            }
        }
    }

    /// Add new terminals that must be reached. Typically those will be requirements based on the
    /// resolvers that were chosen.
    pub(crate) fn extend_terminals(&mut self, extra_terminals: impl IntoIterator<Item = G::NodeId>) {
        self.missing_terminals.extend(
            extra_terminals
                .into_iter()
                .map(|node| self.steiner_graph.to_node_ix(node)),
        );
    }

    /// Forces all cost updates to be applied.
    pub(crate) fn apply_all_cost_updates(&mut self) -> bool {
        if !self.cost_update.records.is_empty() {
            if self.cost_update.increases_cost {
                for (_, edge, cost) in self.cost_update.records.drain(..) {
                    self.steiner_graph.graph[edge] = cost;
                }
                self.regenerate_shortest_paths();
            } else {
                debug_assert!(self.tmp.lowered_cost_nodes.is_empty());
                for (source, edge, cost) in self.cost_update.records.drain(..) {
                    self.steiner_graph.graph[edge] = cost;
                    self.tmp.lowered_cost_nodes.push(source);
                }
                self.steiner_tree
                    .update_shortest_paths(&self.steiner_graph, &mut self.tmp.lowered_cost_nodes);
            }
            true
        } else {
            false
        }
    }

    fn regenerate_shortest_paths(&mut self) {
        for shortest_path in &mut self.steiner_tree.shortest_paths {
            shortest_path.cumulative_cost_from_root = Cost::MAX;
        }
        self.steiner_tree.shortest_paths[self.root_ix.index()].cumulative_cost_from_root = 0;

        debug_assert!(self.tmp.lowered_cost_nodes.is_empty());
        self.tmp.lowered_cost_nodes.push(self.root_ix);
        self.steiner_tree
            .update_shortest_paths(&self.steiner_graph, &mut self.tmp.lowered_cost_nodes);
    }

    /// Core function that actually moves forward the algorithm. At each step we include at least
    /// one new terminal with a non-zero cost.
    /// The control is given back to the caller allowing for any edge cost updates as the cost of requirements might have changed.
    /// The return value indicates whether we still have any missing terminals left.
    pub(crate) fn continue_steiner_tree_growth(&mut self) -> bool {
        // Ensure we start from a clean state.
        self.apply_all_cost_updates();

        tracing::debug!("Grow Steiner Tree:\n{}", self.debug_dot_graph());

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
            &mut self.steiner_graph,
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

            self.steiner_tree.grow_with_some_terminals(
                &mut self.steiner_graph,
                &mut all_zero_cost_terminals,
                &mut self.tmp,
            );
        }

        !self.missing_terminals.is_empty()
    }

    // Estimate the extra cost of retrieving additional terminals with the current Steiner tree and
    // a few edges force set to zero cost.
    //
    // Used to estimate the cost of all requirements of a resolver when taking a certain path.
    pub(crate) fn estimate_extra_cost(
        &mut self,
        zero_cost_edges: impl IntoIterator<Item = G::EdgeId>,
        extra_terminals: impl IntoIterator<Item = G::NodeId>,
    ) -> Cost {
        debug_assert!(self.tmp.terminals_buffer.is_empty());
        self.tmp.terminals_buffer.extend(
            extra_terminals
                .into_iter()
                .map(|node| self.steiner_graph.to_node_ix(node))
                .filter(|node_ix| !self.steiner_tree.nodes[node_ix.index()]),
        );

        if self.tmp.terminals_buffer.is_empty() {
            return 0;
        }

        debug_assert!(self.tmp.lowered_cost_nodes.is_empty() && self.tmp.cost_backup.is_empty());
        for edge_id in zero_cost_edges {
            let edge_ix = self.steiner_graph.to_edge_ix(edge_id);
            let edge_cost = &mut self.steiner_graph.graph[edge_ix];
            if *edge_cost > 0 {
                self.tmp.cost_backup.push((edge_ix, *edge_cost));
                *edge_cost = 0;
                let source_ix = self.steiner_graph.graph.edge_endpoints(edge_ix).unwrap().0;
                self.tmp.lowered_cost_nodes.push(source_ix);
            }
        }

        self.tmp.steiner_tree_copy.clone_from(&self.steiner_tree);
        self.tmp
            .steiner_tree_copy
            .update_shortest_paths(&self.steiner_graph, &mut self.tmp.lowered_cost_nodes);
        self.tmp
            .terminals_buffer
            .sort_by_key(|t| self.tmp.steiner_tree_copy.node_addition_cost(*t));
        self.tmp
            .steiner_tree_copy
            .grow_with_all_missing_terminals(&self.steiner_graph, &mut self.tmp.terminals_buffer);

        self.tmp.terminals_buffer.clear();
        for (edge_ix, cost) in self.tmp.cost_backup.drain(..) {
            self.steiner_graph.graph[edge_ix] = cost;
        }

        self.tmp.steiner_tree_copy.cost - self.steiner_tree.cost
    }

    // Whether a node is part of the Steiner tree.
    pub(crate) fn contains_node(&self, node_id: G::NodeId) -> bool {
        self.steiner_tree.nodes[self.steiner_graph.to_node_ix(node_id).index()]
    }

    #[cfg(test)]
    pub(crate) fn total_cost(&self) -> Cost {
        self.steiner_tree.cost
    }
}

#[derive(Clone)]
struct ShortestPathSteinerTree {
    nodes: fixedbitset::FixedBitSet,
    shortest_paths: Vec<ShortestPath>,
    cost: Cost,
}

#[derive(Clone)]
struct ShortestPath {
    parent: NodeIndex,
    incoming_edge_ix: EdgeIndex,
    cumulative_cost_from_root: Cost,
}

impl ShortestPathSteinerTree {
    pub(crate) fn grow_with_some_terminals<G: GraphBase>(
        &mut self,
        ctx: &mut SteinerGraph<G>,
        terminals: &mut [NodeIndex],
        tmp: &mut Reusable,
    ) {
        debug_assert!(terminals.is_sorted_by_key(|t| self.node_addition_cost(*t)));

        debug_assert!(tmp.lowered_cost_nodes.is_empty());
        for node in terminals {
            let mut node = *node;
            while !self.nodes.put(node.index()) {
                let shortest_path = &self.shortest_paths[node.index()];
                let edge_cost = &mut ctx.graph[shortest_path.incoming_edge_ix];
                if *edge_cost > 0 {
                    self.cost = self.cost.saturating_add(*edge_cost);
                    *edge_cost = 0;
                    tmp.lowered_cost_nodes.push(shortest_path.parent);
                }
                node = shortest_path.parent;
            }
        }

        self.update_shortest_paths(ctx, &mut tmp.lowered_cost_nodes);
    }

    /// Contrary to the grow_with_some_terminals method, this one assumes that we'll never consider
    /// further terminals afterwards. As such we skip the shortest path update.
    ///
    /// Used when estimating the extra cost of terminals as we're manipulating a temporary copy of
    /// the Steiner tree.
    pub(crate) fn grow_with_all_missing_terminals<G: GraphBase>(
        &mut self,
        ctx: &SteinerGraph<G>,
        terminals: &mut [NodeIndex],
    ) {
        debug_assert!(terminals.is_sorted_by_key(|t| self.node_addition_cost(*t)));

        for node in terminals {
            let mut node = *node;
            while !self.nodes.put(node.index()) {
                let shortest_path = &self.shortest_paths[node.index()];
                self.cost = self.cost.saturating_add(ctx.graph[shortest_path.incoming_edge_ix]);
                node = shortest_path.parent;
            }
        }
    }

    /// Updates the shortest paths for all nodes in the graph. We assume the graph to be a DAG and
    /// that cost from the "source" nodes can only decrease.
    fn update_shortest_paths<G: GraphBase>(&mut self, ctx: &SteinerGraph<G>, sources: &mut Vec<NodeIndex>) {
        while let Some(parent) = sources.pop() {
            let parent_cumulative_cost_from_root = self.node_addition_cost(parent);
            for edge in ctx.graph.edges(parent) {
                let node = edge.target();
                let cumulative_cost_from_root = parent_cumulative_cost_from_root.saturating_add(*edge.weight());
                if cumulative_cost_from_root < self.shortest_paths[node.index()].cumulative_cost_from_root {
                    sources.push(node);
                    self.shortest_paths[node.index()] = ShortestPath {
                        parent,
                        incoming_edge_ix: edge.id(),
                        cumulative_cost_from_root,
                    };
                }
            }
        }
    }

    /// Returns the total cost of a node if we were to add it to the Steiner tree.
    fn node_addition_cost(&self, node: NodeIndex) -> Cost {
        self.shortest_paths[node.index()].cumulative_cost_from_root
    }
}
