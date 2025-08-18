use crate::{
    dot_graph::Attrs,
    solve::input::{SteinerEdgeId, SteinerNodeId, SteinerWeight},
};
use fixedbitset::FixedBitSet;
use fxhash::FxBuildHasher;
use petgraph::{
    Graph,
    graph::{EdgeIndex, EdgeReference, NodeIndex},
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

struct Flow {
    saturated_edges: FixedBitSet,
    marked_or_saturated_edges: FixedBitSet,
    root_feeding_terminals: FixedBitSet,
    node_to_feeding_terminals: Vec<FixedBitSet>,
    node_to_flow_rates: Vec<FlowRate>,
}

#[allow(non_snake_case)]
pub(crate) struct GreedyFlac {
    flow: Flow,
    // Run state, re-used across each run
    time: Time,
    heap: PriorityQueue<SteinerEdgeId, Priority, FxBuildHasher>,
    tmp_stack: Vec<NodeIndex>,
    tmp_next_saturating_edges_in_T_u: Vec<NextSaturatingEdge>,
}

struct NextSaturatingEdge {
    edge_id: SteinerEdgeId,
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
    fn run(&mut self) -> ControlFlow<()> {
        if !self.initialize_terminals() {
            // No terminals to process, nothing to do
            return ControlFlow::Break(());
        }

        // Run the algorithm
        tracing::trace!("FLAC:\n{}", self.debug_dot_graph());
        while let Some(edge) = self.get_next_saturating_edge() {
            // The new update_flow_rates handles degenerate flow checking internally
            match self.update_flow_rates(edge) {
                FlacControlFlow::Break => break,
                FlacControlFlow::Continue => continue,
                FlacControlFlow::FoundSubtree { subtree_root_node_id } => {
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
        // last run.
        let n_terminals = self.steiner_tree.terminals.len();
        self.state.flow.root_feeding_terminals.grow(n_terminals);
        let mut has_one_terminal = false;
        for ix in self.state.flow.root_feeding_terminals.zeroes() {
            let terminal = self.steiner_tree.terminals[ix];
            has_one_terminal = true;
            if let Some(edge) = self.find_next_edge_in_T_minus(terminal) {
                let saturate_time = self.time + *edge.weight() as Time;
                self.state.heap.push(edge.id(), saturate_time.into());
                let feeding = &mut self.state.flow.node_to_feeding_terminals[terminal.index()];
                feeding.grow(n_terminals);
                feeding.insert(ix);
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

    #[allow(non_snake_case)]
    fn find_next_edge_in_T_minus(&self, node: NodeIndex) -> Option<EdgeReference<'g, SteinerWeight>> {
        let mut min_edge = None;
        let mut min_weight = SteinerWeight::MAX;

        for edge in self.graph.edges_directed(node, petgraph::Direction::Incoming) {
            if !self.flow.marked_or_saturated_edges.contains(edge.id().index()) {
                let weight = *edge.weight();
                if weight < min_weight {
                    min_weight = weight;
                    min_edge = Some(edge);
                }
            }
        }

        min_edge
    }

    /// Updates flow rates and schedules new edges after a saturated edge is added to the tree.
    ///
    /// When edge (u,v) saturates and is added to the tree, this function:
    /// 1. Updates the path information to record the new connection
    /// 2. Traverses all nodes reachable from u through saturated edges
    /// 3. For each reachable node, checks for degenerate flow and collects min incoming edges
    /// 4. Updates flow rates and schedules new edges after traversal completes
    fn update_flow_rates(&mut self, saturating_edge: EdgeIndex) -> FlacControlFlow {
        // (source, destination)
        let (u, v) = self.graph.edge_endpoints(saturating_edge).unwrap();

        // The current edge will be either saturated or marked
        self.flow.marked_or_saturated_edges.insert(saturating_edge.index());

        // If the node was added to the steiner tree since, no need to continue processing it.
        if self.steiner_tree.nodes[v.index()] {
            return FlacControlFlow::Continue;
        }

        if self.steiner_tree.nodes[u.index()] {
            let new_feeding_terminals = &self.state.flow.node_to_feeding_terminals[v.index()];
            return if self.flow.root_feeding_terminals.is_disjoint(new_feeding_terminals) {
                // If we reach a node in the Steiner tree and we're adding fresh terminals, we'll
                // add it to the steiner tree.
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
                // algorithm and return.
                FlacControlFlow::Break
            };
        }

        // Algorithm 9
        // Check if flow would be degenerate and collect edges to update
        match self.detect_generate_flow_and_collect_edges(u, v) {
            IsDegenerate::Yes => {}
            IsDegenerate::No => {
                // debug_assert!(
                //     !next_saturating_edges_in_T_u.is_empty(),
                //     "No further edges found, but still haven't reached the steiner tree?\n{}",
                //     self.debug_dot_graph()
                // );
                self.flow.saturated_edges.insert(saturating_edge.index());

                // Update all the next saturating edges in T_u
                let v_feeding_terminals = std::mem::take(&mut self.flow.node_to_feeding_terminals[v.index()]);
                let extra_flow_rate = self.flow.node_to_flow_rates[v.index()];
                for NextSaturatingEdge {
                    edge_id,
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
                            debug_assert!(
                                next_saturate_time <= current_saturate_time
                                    && Priority::from(next_saturate_time) >= Priority::from(current_saturate_time),
                                "{} < {} ({} => {} at {})",
                                next_saturate_time,
                                current_saturate_time,
                                old_flow_rate,
                                new_flow_rate,
                                time
                            );
                            *priority = Priority::from(next_saturate_time);
                        });
                    }
                }
                self.flow.node_to_feeding_terminals[v.index()] = v_feeding_terminals;
            }
        }

        // Algorithm 8
        if let Some(edge) = self.find_next_edge_in_T_minus(v) {
            let flow_rate = self.flow.node_to_flow_rates[v.index()];
            debug_assert!(
                flow_rate > 0,
                "Flow rate must be positive, how could it be saturated otherwise?\n{}",
                self.debug_dot_graph()
            );
            let saturate_time =
                self.time + (*edge.weight() - self.graph[saturating_edge]) as Time / (flow_rate as Time);
            self.heap.push(edge.id(), saturate_time.into());
        }

        FlacControlFlow::Continue
    }

    /// Traverses saturated subgraph from u, checking for degenerate flow and collecting the next
    /// saturating edges in T_u while we traverse the parents.
    fn detect_generate_flow_and_collect_edges(&mut self, u: NodeIndex, v: NodeIndex) -> IsDegenerate {
        debug_assert!(self.tmp_stack.is_empty() && self.tmp_next_saturating_edges_in_T_u.is_empty());
        self.tmp_stack.push(u);
        let new_feeding = &self.state.flow.node_to_feeding_terminals[v.index()];

        while let Some(current) = self.state.tmp_stack.pop() {
            // Check for degenerate flow
            let current_feeding = &self.flow.node_to_feeding_terminals[current.index()];
            if !new_feeding.is_disjoint(current_feeding) {
                self.tmp_stack.clear();
                return IsDegenerate::Yes; // Degenerate flow detected
            }

            if let Some(edge) = self.find_next_edge_in_T_minus(current) {
                self.state.tmp_next_saturating_edges_in_T_u.push(NextSaturatingEdge {
                    edge_id: edge.id(),
                    weight: *edge.weight(),
                    target: edge.target(),
                })
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
