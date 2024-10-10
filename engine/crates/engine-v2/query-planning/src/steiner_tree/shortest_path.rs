use bitvec::{bitvec, vec::BitVec};
use petgraph::{
    graph::{IndexType, NodeIndex},
    visit::EdgeRef,
    Direction,
};

use crate::Cost;

use super::{ResolverGraph, Solution, SteinerTreeAlg};

/// Very simple Steiner tree solver, similar to what Neo4j uses, without the extra post processing
/// ("re-routing") and the parallelization.
///
/// We compute the shortest path from the root to the terminals and add those incrementally to the
/// Steiner Tree.
#[allow(unused)]
pub(crate) struct ShortestPathAlg<'a, N, E> {
    graph: &'a ResolverGraph<N, E>,
    root: NodeIndex,
    found_terminals: Vec<NodeIndex>,
    missing_terminals: Vec<NodeIndex>,
    steiner_tree_nodes: BitVec,
    total_cost: Cost,
}

impl<'a, N, E> SteinerTreeAlg<'a, N, E> for ShortestPathAlg<'a, N, E> {
    fn init(graph: &'a ResolverGraph<N, E>, root: NodeIndex, terminals: Vec<NodeIndex>) -> Self {
        let mut steiner_tree_nodes = bitvec![false as usize; graph.edge_count()];
        steiner_tree_nodes.set(root.index(), true);
        Self {
            graph,
            root,
            found_terminals: Vec::with_capacity(terminals.len()),
            missing_terminals: terminals,
            steiner_tree_nodes,
            total_cost: 0,
        }
    }

    #[allow(unused)]
    fn grow_steiner_tree<F>(&mut self, edge_cost: F) -> Option<Solution>
    where
        F: Fn(&E) -> Cost,
    {
        // Compute shortest path from the root
        let shortest_paths = compuste_shortest_paths(self.graph, self.root, edge_cost);

        // The heuristic from here on is that the shortest path to a terminal node is very likely to be
        // included in the Steiner tree. So we first order them by smallest cost and then add the paths
        // until have all terminals.
        self.missing_terminals.sort_unstable_by(|a, b| {
            shortest_paths[a.index()]
                .cumulative_cost_from_root
                .cmp(&shortest_paths[b.index()].cumulative_cost_from_root)
        });
        debug_assert!(self
            .missing_terminals
            .iter()
            .all(|t| shortest_paths[t.index()].cumulative_cost_from_root < Cost::MAX));

        // Add all terminals with 0 cost and the first non-zero cost terminal. This ensures we
        // always grow the Steiner tree at each iteration with something. The caller has then an
        // opportunity to change the cost of edges and any requirements to the terminals.
        let partition_point = self
            .missing_terminals
            .partition_point(|terminal| shortest_paths[terminal.index()].cumulative_cost_from_root == 0);

        let end = (partition_point + 1).min(self.missing_terminals.len());
        let mut terminals = self.missing_terminals.split_off(end);
        std::mem::swap(&mut terminals, &mut self.missing_terminals);

        for terminal in terminals {
            let mut node = terminal;
            let mut cost = shortest_paths[terminal.index()].cumulative_cost_from_root;
            // Skip if already present, shouldn't be needed with operation graph, but might for
            // steiner tree tests.
            if self.steiner_tree_nodes.replace(terminal.index(), true) {
                continue;
            }
            loop {
                let parent = shortest_paths[node.index()].parent;
                // If the parent is already part of the Steiner tree, we stop an only add the
                // extra cost to the terminal.
                if self.steiner_tree_nodes.replace(parent.index(), true) {
                    cost -= shortest_paths[parent.index()].cumulative_cost_from_root;
                    break;
                }
                if parent == self.root {
                    break;
                }
                node = parent
            }
            self.found_terminals.push(terminal);
            self.total_cost += cost;
        }

        if self.missing_terminals.is_empty() {
            Some(Solution {
                terminals: std::mem::take(&mut self.found_terminals),
                steiner_tree_nodes: std::mem::take(&mut self.steiner_tree_nodes),
                total_cost: self.total_cost,
            })
        } else {
            None
        }
    }

    fn steiner_tree_nodes(&self) -> &BitVec {
        &self.steiner_tree_nodes
    }

    fn missing_terminals(&mut self) -> &mut Vec<NodeIndex> {
        &mut self.missing_terminals
    }
}

#[derive(Clone)]
struct ShortestPath {
    parent: NodeIndex,
    cumulative_cost_from_root: Cost,
}

/// This assumes that the graph is a DAG.
fn compuste_shortest_paths<N, E, F>(graph: &ResolverGraph<N, E>, root: NodeIndex, edge_cost: F) -> Vec<ShortestPath>
where
    F: Fn(&E) -> Cost,
{
    let mut parents = vec![
        ShortestPath {
            parent: root,
            cumulative_cost_from_root: Cost::MAX
        };
        graph.node_count()
    ];
    parents[root.index()] = ShortestPath {
        parent: root,
        cumulative_cost_from_root: 0,
    };

    let mut stack = vec![root];
    while let Some(parent) = stack.pop() {
        for edge in graph.edges_directed(parent, Direction::Outgoing) {
            let node = edge.target();
            stack.push(node);
            let cost = parents[parent.index()].cumulative_cost_from_root + edge_cost(edge.weight());
            if cost < parents[node.index()].cumulative_cost_from_root {
                parents[node.index()] = ShortestPath {
                    parent: edge.source(),
                    cumulative_cost_from_root: cost,
                };
            }
        }
    }

    parents
}
