use petgraph::{
    graph::GraphIndex,
    visit::{EdgeRef, GraphBase, NodeCount, NodeIndexable},
};

use crate::{Cost, solve::steiner_tree::SteinerGraph};

use super::{Reusable, SteinerContext};

#[derive(Clone)]
pub(super) struct ShortestPathSteinerTree<G: GraphBase<NodeId: Copy, EdgeId: Copy>> {
    steiner_tree_nodes: fixedbitset::FixedBitSet,
    shortest_path_incoming_edge: Vec<G::EdgeId>,
    shortest_path_costs: Vec<Cost>,
    pub(super) cost: Cost,
}

// Using generics allows us to more confident that the G::NodeId we receive is really the one from
// the graph we used to initialize the shortest paths with.
impl<G: GraphBase<NodeId: GraphIndex> + NodeCount + NodeIndexable> ShortestPathSteinerTree<G> {
    pub(super) fn contains(&self, node: G::NodeId) -> bool {
        self.steiner_tree_nodes[node.index()]
    }

    pub(super) fn build(ctx: &SteinerContext<impl GraphBase, G>) -> Self {
        ShortestPathSteinerTree {
            steiner_tree_nodes: {
                let mut nodes = fixedbitset::FixedBitSet::with_capacity(ctx.graph.node_bound());
                nodes.put(ctx.root_ix.index());
                nodes
            },
            shortest_path_incoming_edge: vec![ctx.incoming_root_edge; ctx.graph.node_count()],
            shortest_path_costs: {
                let mut costs = vec![Cost::MAX; ctx.graph.node_count()];
                costs[ctx.root_ix.index()] = 0;
                costs
            },
            cost: 0,
        }
    }

    pub(super) fn reset_shortest_paths(&mut self, ctx: &SteinerContext<impl GraphBase, G>) {
        for cost in &mut self.shortest_path_costs {
            *cost = Cost::MAX;
        }
        self.shortest_path_costs[ctx.root_ix.index()] = 0;
    }

    pub(super) fn node_addition_cost(&self, node: G::NodeId) -> Cost {
        self.shortest_path_costs[node.index()]
    }
}

impl ShortestPathSteinerTree<SteinerGraph> {
    pub(crate) fn grow_with_some_terminals(
        &mut self,
        ctx: &mut SteinerContext<impl GraphBase, SteinerGraph>,
        terminals: &mut [<SteinerGraph as GraphBase>::NodeId],
        tmp: &mut Reusable<SteinerGraph>,
    ) {
        debug_assert!(terminals.is_sorted_by_key(|t| self.node_addition_cost(*t)));

        debug_assert!(tmp.lowered_cost_nodes.is_empty());
        for node in terminals {
            let mut node = *node;
            while !self.steiner_tree_nodes.put(node.index()) {
                let incoming_edge_ix = self.shortest_path_incoming_edge[node.index()];
                let parent = ctx.graph.edge_endpoints(incoming_edge_ix).unwrap().0;
                let edge_cost = &mut ctx.graph[incoming_edge_ix];
                if *edge_cost > 0 {
                    self.cost = self.cost.saturating_add(*edge_cost);
                    *edge_cost = 0;
                    tmp.lowered_cost_nodes.push(parent);
                }
                node = parent;
            }
        }

        self.update_shortest_paths(ctx, &mut tmp.lowered_cost_nodes);
    }

    /// Contrary to the grow_with_some_terminals method, this one assumes that we'll never consider
    /// further terminals afterwards. As such we skip the shortest path update.
    ///
    /// Used when estimating the extra cost of terminals as we're manipulating a temporary copy of
    /// the Steiner tree.
    pub(super) fn grow_with_all_missing_terminals(
        &mut self,
        ctx: &SteinerContext<impl GraphBase, SteinerGraph>,
        terminals: &mut [<SteinerGraph as GraphBase>::NodeId],
    ) {
        debug_assert!(terminals.is_sorted_by_key(|t| self.node_addition_cost(*t)));

        for node in terminals {
            let mut node = *node;
            while !self.steiner_tree_nodes.put(node.index()) {
                let incoming_edge_ix = self.shortest_path_incoming_edge[node.index()];
                self.cost = self.cost.saturating_add(ctx.graph[incoming_edge_ix]);
                node = ctx.graph.edge_endpoints(incoming_edge_ix).unwrap().0;
            }
        }
    }

    /// Updates the shortest paths for all nodes in the graph. We assume the graph to be a DAG and
    /// that cost from the "source" nodes can only decrease.
    pub(super) fn update_shortest_paths(
        &mut self,
        ctx: &SteinerContext<impl GraphBase, SteinerGraph>,
        sources: &mut Vec<<SteinerGraph as GraphBase>::NodeId>,
    ) {
        while let Some(parent) = sources.pop() {
            let parent_cumulative_cost_from_root = self.shortest_path_costs[parent.index()];
            for edge in ctx.graph.edges(parent) {
                let node = edge.target();
                let cumulative_cost_from_root = parent_cumulative_cost_from_root.saturating_add(*edge.weight());
                let current = &mut self.shortest_path_costs[node.index()];
                if cumulative_cost_from_root < *current {
                    sources.push(node);
                    *current = cumulative_cost_from_root;
                    self.shortest_path_incoming_edge[node.index()] = edge.id();
                }
            }
        }
    }
}
