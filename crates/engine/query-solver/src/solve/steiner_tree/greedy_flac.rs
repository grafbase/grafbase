use crate::{
    dot_graph::Attrs,
    solve::input::{SteinerEdgeId, SteinerNodeId, SteinerWeight},
};
use fixedbitset::FixedBitSet;
use fxhash::FxBuildHasher;
use petgraph::{
    Graph,
    graph::{EdgeIndex, NodeIndex},
    visit::{EdgeIndexable, EdgeRef, NodeIndexable},
};
use priority_queue::PriorityQueue;
use std::{cmp::Ordering, ops::ControlFlow};

use super::SteinerTree;

type Time = f64;
type FlowRate = SteinerWeight;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Priority(Time);

impl From<Time> for Priority {
    fn from(time: Time) -> Self {
        Priority(time)
    }
}

impl From<Priority> for Time {
    fn from(priority: Priority) -> Self {
        priority.0
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0).reverse()
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Priority {}

/// # GreedyFLAC
///
/// Watel, D., & Weisser, M. A. (2016). A practical greedy approximation for the directed steiner tree problem. Journal of Combinatorial Optimization, 32(4), 1327-1370.
/// https://www.researchgate.net/profile/Dimitri-Watel/publication/307916063_A_practical_greedy_approximation_for_the_directed_Steiner_tree_problem/links/5f04a382299bf18816082829/A-practical-greedy-approximation-for-the-directed-Steiner-tree-problem.pdf
///
/// ## Overview
///
/// The Query Solver uses the GreedyFLAC algorithm to find a good query resolution paths in GraphQL federation. This algorithm solves the Directed Steiner Tree problem - finding the minimum-cost tree that connects a root node to all required terminal nodes in a directed graph.
///
/// ## The Water Flow Analogy
///
/// FLAC (FLow Algorithm Computation) uses an intuitive water flow metaphor to build the Steiner tree:
///
/// 1. **Terminals as Water Sources**: Each terminal node acts as a water source, continuously pouring water at 1 unit/second
/// 2. **Edges as Pipes**: Each edge has a capacity equal to its cost/weight
/// 3. **Saturation**: When an edge is completely filled with water, it becomes "saturated" and part of the solution
/// 4. **Flow Propagation**: Water flows backward through the graph (from terminals toward the root) until reaching the root
///
/// The GreedyFLAC simply applies the FLAC algorithm as many times as necessary to build a steiner tree with all terminals.
///
/// ## Simple Example
///
/// Consider finding the cheapest way to connect a root server to three data terminals (T1, T2, T3):
///
/// ```
///         Root
///        /    \
///     $5/      \$8
///      /        \
///     A          B
///    / \          \
/// $2/   \$3        \$7
///  /     \          \
/// T1      T2        T3
/// ```
///
/// ### How GreedyFLAC Solves This:
///
/// **Step 1: Initialize Flow**
///
/// - T1, T2, and T3 each start flowing water at 1 unit/second
/// - T1 flows into edge (A→T1) with cost $2
/// - T2 flows into edge (A→T2) with cost $3
/// - T3 flows into edge (B→T3) with cost $7
///
/// ```
///         Root
///        /    \
///     $5/      \$8
///      /        \
///     A          B
///    / \          \
/// $2/   \$3        \$7
///  ↑     ↑          ↑
/// T1     T2        T3
/// (1/s)  (1/s)     (1/s)
/// ```
///
/// **Step 2: First Saturation (t=2s)**
///
/// - Edge (A→T1) saturates after 2 seconds (2 units filled)
/// - A now has flow rate of 1 unit/second from T1
/// - Water from T1 continues through A's incoming edge (Root→A)
///
/// ```
///         Root
///        /    \
///     $5/      \$8
///      ↑        \
///     A(1/s)     B
///    /=\          \
/// $2/   \$3        \$7
///  /     ↑          ↑
/// T1     T2        T3
///        (1/s)     (1/s)
///
/// Legend: = saturated edge
/// ```
///
/// **Step 3: Second Saturation (t=3s)**
///
/// - Edge (A→T2) saturates after 3 seconds
/// - A now has flow rate of 2 units/second (from T1 and T2)
/// - Water flows faster through (Root→A)
///
/// ```
///         Root
///        /    \
///     $5/      \$8
///      ↑        \
///     A(2/s)     B
///    /=\=         \
/// $2/   \$3        \$7
///  /     \          ↑
/// T1     T2        T3
///                  (1/s)
/// ```
///
/// **Step 4: Root Reached (t=4.5s)**
///
/// - Edge (Root→A) saturates after 4.5 seconds total
///   - First 2 seconds: 1 unit/sec × 2 sec = 2 units
///   - Next 1.5 seconds: 2 units/sec × 1.5 sec = 3 units
///   - Total: 5 units (edge capacity)
/// - Root reached! Partial tree for T1 and T2 complete
///
/// ```
///         Root
///        /====\
///     $5/      \$8
///      /        \
///     A          B
///    /=\=         \
/// $2/   \$3        \$7
///  /     \          ↑
/// T1     T2        T3
///                  (1/s)
/// ```
/// **Step 5: Greed **
///
/// Once we found a partial tree, re-iterate with the FLAC algorithm as many times as necessary to include all terminals.
///
/// ## Degenerate Flow Prevention
///
/// A critical aspect of GreedyFLAC is preventing "degenerate flow" - situations where water from the same terminal reaches a node through multiple paths. This would create cycles in our tree, which we must avoid.
///
/// ### Example of Degenerate Flow:
///
/// ```
///      Root
///        |
///       $10
///        |
///        A
///       / \
///    $2/   \$3
///     /     \
///    B       C
///     \     /
///    $1\   /$1
///       \ /
///        T1
/// ```
///
/// Without degenerate flow detection:
///
/// 1. T1 would flow into both (B→T1) and (C→T1)
/// 2. Both edges would saturate after 1 second
/// 3. Both B and C would start flowing toward A
/// 4. When A is reached, it would receive flow from the same terminal (T1) via two different paths
/// 5. This creates a cycle, not a tree!
///
/// **Solution**: When the algorithm detects that adding an edge would create degenerate flow (same terminal reaching a node through multiple paths), it marks that edge as forbidden and continues with alternative paths. This ensures the result is always a valid tree structure.
///
/// In the example above, the algorithm would:
///
/// 1. Allow one path (e.g., A→B→T1) to saturate normally
/// 2. Detect that allowing A→C→T1 would create degenerate flow
/// 3. Mark edge (C→T1) as forbidden
/// 4. Find an alternative solution that maintains the tree property
///
/// This guarantees that the final Steiner tree has no cycles and each terminal is connected to the root through exactly one path.
#[allow(non_snake_case)]
pub(crate) struct GreedyFlac {
    flow: Flow,
    // Run state, re-used across each run
    time: Time,
    heap: PriorityQueue<SteinerEdgeId, Priority, FxBuildHasher>,
    tmp_stack: Vec<NodeIndex>,
    tmp_next_saturating_edges_in_T_u: Vec<NextSaturatingEdge>,
}

struct Flow {
    saturated_edges: FixedBitSet,
    marked_or_saturated_edges: FixedBitSet,
    root_feeding_terminals: FixedBitSet,
    node_to_feeding_terminals: Vec<FixedBitSet>,
    node_to_flow_rates: Vec<FlowRate>,
}

struct NextSaturatingEdge {
    id: SteinerEdgeId,
    weight: SteinerWeight,
    target: SteinerNodeId,
}

impl GreedyFlac {
    pub fn new<N>(graph: &Graph<N, SteinerWeight>) -> Self {
        Self {
            flow: Flow {
                saturated_edges: FixedBitSet::with_capacity(graph.edge_bound()),
                marked_or_saturated_edges: FixedBitSet::with_capacity(graph.edge_bound()),
                root_feeding_terminals: FixedBitSet::new(),
                node_to_feeding_terminals: vec![FixedBitSet::new(); graph.node_bound()],
                node_to_flow_rates: vec![0; graph.node_bound()],
            },
            time: 0.0,
            heap: PriorityQueue::default(),
            // Re-used allocations
            tmp_stack: Vec::with_capacity(32),
            tmp_next_saturating_edges_in_T_u: Vec::with_capacity(16),
        }
    }

    pub fn run_once<N>(&mut self, graph: &Graph<N, SteinerWeight>, steiner_tree: &mut SteinerTree) -> ControlFlow<()>
    where
        N: std::fmt::Debug,
    {
        Flac {
            state: self,
            graph,
            steiner_tree,
        }
        .run()
    }

    pub fn run<N>(&mut self, graph: &Graph<N, SteinerWeight>, steiner_tree: &mut SteinerTree)
    where
        N: std::fmt::Debug,
    {
        let mut flac = Flac {
            state: self,
            graph,
            steiner_tree,
        };
        while flac.run().is_continue() {}
    }

    pub fn reset(&mut self) {
        self.flow.root_feeding_terminals.clear();
    }

    #[cfg(test)]
    pub fn debug_dot_graph<N>(&self, graph: &Graph<N, SteinerWeight>, steiner_tree: &SteinerTree) -> String
    where
        N: std::fmt::Debug,
    {
        debug_dot_graph(self, graph, steiner_tree)
    }
}

struct Flac<'s, 'g, 't, N> {
    state: &'s mut GreedyFlac,
    graph: &'g Graph<N, SteinerWeight>,
    steiner_tree: &'t mut SteinerTree,
}

impl<N> std::ops::Deref for Flac<'_, '_, '_, N> {
    type Target = GreedyFlac;
    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<N> std::ops::DerefMut for Flac<'_, '_, '_, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

impl<'g, N> Flac<'_, 'g, '_, N>
where
    N: std::fmt::Debug,
{
    /// Run the FLAC algorithm.
    ///
    /// # Returns
    /// - `Break` if all terminals are connected or no more progress can be made
    /// - `Continue` if more iterations are needed to connect all terminals
    fn run(&mut self) -> ControlFlow<()> {
        if !self.initialize_terminals() {
            // No terminals to process, nothing to do
            return ControlFlow::Break(());
        }

        // Run the water flow simulation
        tracing::trace!("FLAC:\n{}", self.debug_dot_graph());
        while let Some(edge) = self.get_next_saturating_edge() {
            // Process the saturated edge and update flow rates
            match self.update_flow_rates(edge) {
                FlacControlFlow::Break => break,
                FlacControlFlow::Continue => continue,
                FlacControlFlow::FoundSubtree { subtree_root_node_id } => {
                    // Found a path from terminals to the existing Steiner tree
                    self.steiner_tree.total_weight += self.graph[edge];
                    self.steiner_tree.edges.insert(edge.index());

                    // We traverse in the opposite direction to FLAC as not all saturated edges from
                    // the terminals lead to anywhere useful. The algorithm stops at the first path
                    // that leads to an existing node of the Steiner Tree.
                    debug_assert!(self.tmp_stack.is_empty());
                    self.tmp_stack.push(subtree_root_node_id);
                    while let Some(node) = self.tmp_stack.pop() {
                        self.steiner_tree.nodes.insert(node.index());
                        for edge in self.graph.edges_directed(node, petgraph::Direction::Outgoing) {
                            if self.flow.saturated_edges[edge.id().index()] {
                                self.steiner_tree.edges.insert(edge.id().index());
                                self.steiner_tree.total_weight += *edge.weight();
                                self.tmp_stack.push(edge.target());
                            }
                        }
                    }
                }
            }
            tracing::trace!("FLAC:\n{}", self.debug_dot_graph());
        }

        if self.flow.root_feeding_terminals.is_full() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    /// Initializes the water flow simulation for terminals not yet connected to the tree. All of
    /// them start with 1 flow rate and we reset the state of nodes & edges.
    fn initialize_terminals(&mut self) -> bool {
        self.time = 0.0;
        self.heap.clear();
        self.flow.saturated_edges.clear();
        self.flow.marked_or_saturated_edges.clear();
        self.flow
            .node_to_feeding_terminals
            .iter_mut()
            .for_each(|set| set.clear());
        self.flow.node_to_flow_rates.fill(0);

        // Prepare the initial state
        debug_assert!(
            !self.steiner_tree.nodes.is_empty(),
            "Root must be part of the steiner tree."
        );
        debug_assert!(self.tmp_stack.is_empty() && self.tmp_next_saturating_edges_in_T_u.is_empty());

        // Initialize the state with the current terminals. New ones may have been added since the
        // last run (due to requirements becoming mandatory).
        let n_terminals = self.steiner_tree.terminals.len();
        self.state.flow.root_feeding_terminals.grow(n_terminals);
        let mut has_one_terminal = false;
        for ix in self.state.flow.root_feeding_terminals.zeroes() {
            let terminal = self.steiner_tree.terminals[ix];
            has_one_terminal = true;
            if let Some(edge) = self.find_next_edge_in_T_minus(terminal) {
                // Schedule when this edge will saturate based on its weight (capacity)
                let saturate_time = self.time + edge.weight as Time;
                self.state.heap.push(edge.id, saturate_time.into());
                // Track that this terminal is feeding water to itself
                let feeding = &mut self.state.flow.node_to_feeding_terminals[terminal.index()];
                feeding.grow(n_terminals);
                feeding.insert(ix);
                // Set initial flow rate of 1 unit/second
                self.state.flow.node_to_flow_rates[terminal.index()] = 1;
            }
        }

        has_one_terminal
    }

    fn get_next_saturating_edge(&mut self) -> Option<EdgeIndex> {
        let (edge, priority) = self.heap.pop()?;
        self.time = priority.into();
        Some(edge)
    }

    /// In the original paper, they suggest keep a sorted list of edges, but that's way too
    /// expensive to keep track of for every node. Instead we just re-compute the next edge that
    /// will saturate each time.
    #[allow(non_snake_case)]
    fn find_next_edge_in_T_minus(&self, node: NodeIndex) -> Option<NextSaturatingEdge> {
        let mut min_edge = usize::MAX;
        let mut min_weight = SteinerWeight::MAX;
        let mut target = usize::MAX;

        for edge in self.graph.edges_directed(node, petgraph::Direction::Incoming) {
            let edge_index = edge.id().index();
            let weight = *edge.weight();
            // SAFETY: Guaranteed to be the right size by the assert in the initialization.
            let is_min =
                 !self.flow.marked_or_saturated_edges[edge_index] & (weight < min_weight);
            // if marked or saturated -> 1111_1111
            let is_min_weight_mask = (!is_min as SteinerWeight).wrapping_sub(1);
            let is_min_edge_mask = (!is_min as usize).wrapping_sub(1);

            min_weight = (is_min_weight_mask & weight) | (!is_min_weight_mask & min_weight);
            min_edge = (is_min_edge_mask & edge_index) | (!is_min_edge_mask & min_edge);
            target = (is_min_edge_mask & edge.target().index()) | (!is_min_edge_mask & target);
        }

        if min_edge == usize::MAX {
            None
        } else {
            Some(NextSaturatingEdge {
                id: EdgeIndex::new(min_edge),
                weight: min_weight,
                target: NodeIndex::new(target),
            })
        }
    }

    /// A saturating edge can end up in one of two situations in the FLAC
    /// algorithm:
    /// - It saturates and water continues flowing into its source node. This edge may be used in
    ///   steiner tree.
    /// - It leads to degenerate flow if the source node already receives water from one the
    ///   terminal through a different path. To avoid a cycle we mark this edge and ignore it.
    ///
    /// From there we need to update the state of the algorithm.
    fn update_flow_rates(&mut self, saturating_edge: EdgeIndex) -> FlacControlFlow {
        // (source, destination)
        let (u, v) = self.graph.edge_endpoints(saturating_edge).unwrap();

        // The current edge will be either saturated or marked
        self.flow.marked_or_saturated_edges.insert(saturating_edge.index());

        // If the node was added to the steiner tree since, no need to continue processing it.
        // This may happen if we have a first subtree and continue the FLAC algorithm. In that case
        // we might still have leftover edges in the heap from that subtree that should be ignored.
        if self.steiner_tree.nodes[v.index()] {
            return FlacControlFlow::Continue;
        }

        // If the current edge reaches the current steiner tree, we check whether this would lead
        // to a degenerate flow or not. If not we can add this edge and all of it saturated edges
        // to the steiner tree.
        // In the original paper, that's where the FLAC algorithm stops so it doesn't need to do
        // check on the terminals.
        if self.steiner_tree.nodes[u.index()] {
            let new_feeding_terminals = &self.state.flow.node_to_feeding_terminals[v.index()];
            return if self.flow.root_feeding_terminals.is_disjoint(new_feeding_terminals) {
                tracing::trace!(
                    "Found new subtree from {:?} {:b} {:b}",
                    self.graph[v],
                    new_feeding_terminals,
                    self.flow.root_feeding_terminals
                );
                self.state.flow.root_feeding_terminals.union_with(new_feeding_terminals);

                FlacControlFlow::FoundSubtree {
                    subtree_root_node_id: v,
                }
            } else {
                // If there is degenerate flow with the steiner tree, we completely stop the
                // algorithm and return. This means that terminals that have already been included
                // in the steiner tree have an impact on the paths we'll choose.
                FlacControlFlow::Break
            };
        }

        // Algorithm 9
        // Check if flow would be degenerate and collect edges to update. If the flow isn't
        // degenerate we'll need to update the flow of every node accessible from `u`, the source
        // of the newly saturated edge.
        match self.detect_generate_flow_and_collect_edges(u, v) {
            IsDegenerate::Yes => {}
            IsDegenerate::No => {
                self.flow.saturated_edges.insert(saturating_edge.index());

                // Update all the next saturating edges in T_u
                let v_feeding_terminals = std::mem::take(&mut self.flow.node_to_feeding_terminals[v.index()]);
                let extra_flow_rate = self.flow.node_to_flow_rates[v.index()];
                for NextSaturatingEdge {
                    id: edge_id,
                    weight,
                    target,
                } in self.state.tmp_next_saturating_edges_in_T_u.drain(..)
                {
                    let ix = target.index();

                    // Algorithm 5
                    self.state.flow.node_to_feeding_terminals[ix].union_with(&v_feeding_terminals);

                    let old_flow_rate = self.state.flow.node_to_flow_rates[ix];
                    let new_flow_rate = old_flow_rate + extra_flow_rate;
                    self.state.flow.node_to_flow_rates[ix] = new_flow_rate;

                    // Algorithm 7
                    if old_flow_rate == 0 {
                        let saturate_time = self.state.time + (weight as Time / new_flow_rate as Time);
                        self.state.heap.push(edge_id, saturate_time.into());
                    } else {
                        let time = self.state.time;
                        self.state.heap.change_priority_by(&edge_id, |priority| {
                            let current_saturate_time: Time = (*priority).into();
                            let next_saturate_time =
                                time + (current_saturate_time - time) * (old_flow_rate as Time / new_flow_rate as Time);
                            *priority = Priority::from(next_saturate_time);
                        });
                    }
                }
                self.flow.node_to_feeding_terminals[v.index()] = v_feeding_terminals;
            }
        }

        // Algorithm 8
        // The destination node `v` may have other incoming edges. In that case we now need to add
        // the next one into the heap with its saturating time.
        if let Some(edge) = self.find_next_edge_in_T_minus(v) {
            let flow_rate = self.flow.node_to_flow_rates[v.index()];
            debug_assert!(
                flow_rate > 0,
                "Flow rate must be positive, how could it be saturated otherwise?\n{}",
                self.debug_dot_graph()
            );
            let saturate_time = self.time + (edge.weight - self.graph[saturating_edge]) as Time / (flow_rate as Time);
            self.heap.push(edge.id, saturate_time.into());
        }

        FlacControlFlow::Continue
    }

    /// Degenerate flow occurs when water from the same terminal reaches a node through multiple paths.
    /// This would create a cycle in our tree, which we must avoid.
    ///
    /// Detecting whether flow is degenerate must be done by checking all nodes upwards of the
    /// saturating edge. For example if the edge (A, C) saturates in the following graph, we have
    /// to check that `C` doesn't receive flow from either T1 or T2.
    /// ```ignore
    ///       C
    ///      * \
    ///     *   \
    ///    A     B
    ///   / \   /
    ///  /   \ /
    /// T1    T2
    /// ```
    ///
    /// However, if the flow isn't degenerate we'll need to update the flow rate of C, so we end up
    /// traversing the same nodes. This function accumulates all the edges that must be
    /// updated if the flow isn't degenerate until we found them all or we detect a degenerate
    /// flow.
    ///
    fn detect_generate_flow_and_collect_edges(&mut self, u: NodeIndex, v: NodeIndex) -> IsDegenerate {
        debug_assert!(self.tmp_stack.is_empty() && self.tmp_next_saturating_edges_in_T_u.is_empty());
        self.tmp_stack.push(u);
        let new_feeding = &self.state.flow.node_to_feeding_terminals[v.index()];

        while let Some(current) = self.state.tmp_stack.pop() {
            let current_feeding = self.flow.node_to_feeding_terminals[current.index()];
            // Check for degenerate flow
            if !new_feeding.is_disjoint(current_feeding) {
                self.tmp_stack.clear();
                self.tmp_next_saturating_edges_in_T_u.clear();
                return IsDegenerate::Yes; // Degenerate flow detected
            }

            if let Some(edge) = self.find_next_edge_in_T_minus(current) {
                self.state.tmp_next_saturating_edges_in_T_u.push(edge)
            }

            // Add neighbors reachable through saturated edges
            for edge in self.graph.edges_directed(current, petgraph::Direction::Incoming) {
                if self.flow.saturated_edges[edge.id().index()] {
                    let src = edge.source();
                    self.state.tmp_stack.push(src);
                }
            }
        }

        IsDegenerate::No
    }

    fn debug_dot_graph(&self) -> String {
        debug_dot_graph(self, self.graph, self.steiner_tree)
    }
}

enum FlacControlFlow {
    Continue,
    Break,
    FoundSubtree { subtree_root_node_id: SteinerNodeId },
}

#[allow(non_snake_case)]
enum IsDegenerate {
    Yes,
    No,
}

fn debug_dot_graph<N: std::fmt::Debug>(
    greedy_flac: &GreedyFlac,
    graph: &Graph<N, SteinerWeight>,
    steiner_tree: &SteinerTree,
) -> String {
    use petgraph::dot::{Config, Dot};
    let legend = format!(
        r#"
    legend [shape=none, margin=0, label=<
      <table border="1" cellborder="1" cellspacing="0" cellpadding="4">
        <tr><td colspan="2">Time {}</td></tr>
        <tr><td><font color="forestgreen">&#448;</font></td><td>steiner tree</td></tr>
        <tr><td><font color="royalblue">&#448;</font></td><td>saturated</td></tr>
        <tr><td><font color="royalblue">&#119044;</font></td><td>marked</td></tr>
      </table>
    >];"#,
        greedy_flac.time
    );
    format!(
        "digraph {{\n{:?}{legend}\n}}",
        Dot::with_attr_getters(
            &graph,
            &[Config::EdgeNoLabel, Config::NodeNoLabel, Config::GraphContentOnly],
            &|_, edge| {
                let is_in_steiner_tree = steiner_tree.edges[edge.id().index()];
                let is_saturated = greedy_flac.flow.saturated_edges[edge.id().index()];
                let is_marked = greedy_flac.flow.marked_or_saturated_edges[edge.id().index()] && !is_saturated;
                let attr = match (is_in_steiner_tree, is_saturated, is_marked) {
                    (true, _, _) => "color=forestgreen,fontcolor=forestgreen",
                    (_, true, _) => "color=royalblue,fontcolor=royalblue",
                    (_, false, true) => "color=royalblue,fontcolor=royalblue,style=dashed",
                    (_, _, _) => "",
                };

                let label = if *edge.weight() > 0 {
                    let mut label = format!("${}", edge.weight());
                    if let Some(priority) = greedy_flac
                        .heap
                        .iter()
                        .filter_map(|(id, priority)| if *id == edge.id() { Some(*priority) } else { None })
                        .max()
                    {
                        label.push_str(&format!(" at {}", priority.0));
                    }
                    label
                } else {
                    String::new()
                };
                Attrs::label(label).with(attr).to_string()
            },
            &|_, (node_id, _)| {
                let is_terminal = steiner_tree.terminals.contains(&node_id);
                let flow_rate = greedy_flac.flow.node_to_flow_rates[node_id.index()];
                let is_in_steiner_tree = steiner_tree.nodes[node_id.index()];
                let n = graph.edges_directed(node_id, petgraph::Direction::Incoming).count();
                let all_edges_saturated = n > 0
                    && graph
                        .edges_directed(node_id, petgraph::Direction::Incoming)
                        .all(|edge| greedy_flac.flow.saturated_edges[edge.id().index()]);
                let all_edges_saturated_or_marked = n > 0
                    && graph
                        .edges_directed(node_id, petgraph::Direction::Incoming)
                        .all(|edge| greedy_flac.flow.marked_or_saturated_edges[edge.id().index()]);
                let style = if is_in_steiner_tree {
                    "color=forestgreen"
                } else if all_edges_saturated {
                    "color=royalblue"
                } else if all_edges_saturated_or_marked {
                    "color=royalblue,style=dashed"
                } else {
                    ""
                };
                let shape = if is_terminal { "shape=rectangle" } else { "" };
                let mut label = format!("{:?}", &graph[node_id]);
                if label == "()" {
                    label.clear();
                } else {
                    label.push(' ');
                }
                Attrs::label(format!(
                    "<{}{}&#128167;<br/>{:b}>",
                    label,
                    flow_rate,
                    &greedy_flac.flow.node_to_feeding_terminals[node_id.index()],
                ))
                .with(style)
                .with(shape)
                .to_string()
            }
        )
    )
}
