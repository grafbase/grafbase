use crate::{Cost, dot_graph::Attrs};
use fixedbitset::FixedBitSet;
use fxhash::FxBuildHasher;
use itertools::Itertools as _;
use petgraph::{
    Graph,
    graph::{EdgeIndex, EdgeReference, NodeIndex},
    visit::{EdgeRef, NodeIndexable},
};
use priority_queue::PriorityQueue;
use std::{cmp::Ordering, ops::ControlFlow};

use super::SteinerTree;

type Time = f64;
type FlowRate = u16;

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
    terminals: Vec<NodeIndex>,
}

pub(crate) struct GreedyFlac {
    flow: Flow,
    // Run state, re-used across each run
    time: Time,
    heap: PriorityQueue<EdgeIndex, Priority, FxBuildHasher>,
    stack: Vec<NodeIndex>,
}

impl GreedyFlac {
    pub fn new<N>(graph: &Graph<N, Cost>, terminals: Vec<NodeIndex>) -> Self {
        Self {
            flow: Flow {
                saturated_edges: FixedBitSet::with_capacity(graph.edge_count()),
                marked_or_saturated_edges: FixedBitSet::with_capacity(graph.edge_count()),
                root_feeding_terminals: FixedBitSet::new(),
                node_to_feeding_terminals: vec![FixedBitSet::new(); graph.node_bound()],
                node_to_flow_rates: vec![0; graph.node_bound()],
                terminals,
            },
            time: 0.0,
            heap: PriorityQueue::default(),
            stack: Vec::new(),
        }
    }

    pub fn extend_terminals(&mut self, terminals: impl IntoIterator<Item = NodeIndex>) {
        self.flow.terminals.extend(terminals);
    }

    pub fn run_once<N>(&mut self, graph: &Graph<N, Cost>, steiner_tree: &mut SteinerTree) -> ControlFlow<()>
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

    pub fn run<N>(&mut self, graph: &Graph<N, Cost>, steiner_tree: &mut SteinerTree)
    where
        N: std::fmt::Debug,
    {
        let mut runner = Flac {
            state: self,
            graph,
            steiner_tree,
        };
        loop {
            if runner.run().is_break() {
                return;
            }
        }
    }

    pub fn reset_terminals(&mut self) {
        self.flow.root_feeding_terminals.clear();
        self.flow.terminals.clear();
    }
}

struct Flac<'s, 'g, 't, N> {
    state: &'s mut GreedyFlac,
    graph: &'g Graph<N, Cost>,
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
        if self.flow.terminals.is_empty() {
            // No terminals to process, nothing to do
            return ControlFlow::Break(());
        }
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
        debug_assert!(self.stack.is_empty());

        // Initialize the state with the current terminals. New ones may have been added since the
        // last run.
        let n_terminals = self.state.flow.terminals.len();
        self.state.flow.root_feeding_terminals.grow(n_terminals);
        // Can happen if we extend the terminals with nodes that are already part of the steiner
        // tree.
        if self.flow.root_feeding_terminals.is_full() {
            return ControlFlow::Break(());
        }
        for ix in self.state.flow.root_feeding_terminals.zeroes() {
            let terminal = self.state.flow.terminals[ix];
            if let Some(edge) = self.find_next_edge_in_T_minus(terminal) {
                let saturate_time = self.time + *edge.weight() as Time;
                self.state.heap.push(edge.id(), saturate_time.into());
                let feeding = &mut self.state.flow.node_to_feeding_terminals[terminal.index()];
                feeding.grow(n_terminals);
                feeding.insert(ix);
                self.state.flow.node_to_flow_rates[terminal.index()] = 1;
            }
        }

        // Run the algorithm
        loop {
            let Some(edge) = self.get_next_saturating_edge() else {
                unreachable!("Could not reach root?\n{}", self.debug_dot_graph());
            };

            // The new update_flow_rates handles degenerate flow checking internally
            if let ControlFlow::Break((_, v)) = self.update_flow_rates(edge) {
                let new_feeding_terminals = &self.state.flow.node_to_feeding_terminals[v.index()];
                debug_assert!(
                    !new_feeding_terminals.is_clear(),
                    "No new terminals?\n{}",
                    self.debug_dot_graph()
                );
                debug_assert!(
                    (new_feeding_terminals & (&self.flow.root_feeding_terminals)).is_clear(),
                    "New feeding terminals weren't distinct from the current ones. This means older ones were still flowing.\n{}\n{:b}\n{:b}\n{}",
                    self.flow
                        .terminals
                        .iter()
                        .map(|idx| &self.graph[*idx])
                        .format_with(",", |node, f| f(&format_args!("{node:?}"))),
                    &new_feeding_terminals,
                    &self.flow.root_feeding_terminals,
                    self.debug_dot_graph()
                );

                self.state.flow.root_feeding_terminals.union_with(new_feeding_terminals);

                self.steiner_tree.total_weight += self.graph[edge];
                self.steiner_tree.edges.insert(edge.index());

                // We traverse in the opposite direction to FLAC as not all saturated edges from
                // the terminals lead to anywhere useful. The algorithm stops at the first path
                // that leads to an existing node of the Steiner Tree.
                debug_assert!(self.stack.is_empty());
                self.stack.push(v);
                while let Some(node) = self.stack.pop() {
                    self.steiner_tree.nodes.insert(node.index());
                    for edge in self.graph.edges_directed(node, petgraph::Direction::Outgoing) {
                        if self.flow.saturated_edges[edge.id().index()] {
                            self.steiner_tree.edges.insert(edge.id().index());
                            self.steiner_tree.total_weight += *edge.weight();
                            self.stack.push(edge.target());
                        }
                    }
                }

                return if self.flow.root_feeding_terminals.is_full() {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                };
            }
        }
    }

    fn get_next_saturating_edge(&mut self) -> Option<EdgeIndex> {
        let (edge, priority) = self.heap.pop()?;
        self.time = priority.into();
        Some(edge)
    }

    #[allow(non_snake_case)]
    fn find_next_edge_in_T_minus(&self, node: NodeIndex) -> Option<EdgeReference<'g, Cost>> {
        let mut min_edge = None;
        let mut min_cost = Cost::MAX;

        for edge in self.graph.edges_directed(node, petgraph::Direction::Incoming) {
            if !self.flow.marked_or_saturated_edges.contains(edge.id().index()) {
                let cost = *edge.weight();
                if cost < min_cost {
                    min_cost = cost;
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
    fn update_flow_rates(&mut self, saturating_edge: EdgeIndex) -> ControlFlow<(NodeIndex, NodeIndex)> {
        // (source, destination)
        let (u, v) = self.graph.edge_endpoints(saturating_edge).unwrap();

        // The current edge will be either saturated or marked
        self.flow.marked_or_saturated_edges.insert(saturating_edge.index());

        // When the algorithm reaches a node of the Steiner Tree, which starts with only the root node,
        // we don't need to go further.
        if self.steiner_tree.nodes[u.index()] {
            return ControlFlow::Break((u, v));
        }

        // Algorithm 9
        // Check if flow would be degenerate and collect edges to update
        match self.detect_generate_flow_and_collect_edges(u, v) {
            DegenerateFlow::Yes => {}
            DegenerateFlow::No {
                next_saturating_edges_in_T_u,
            } => {
                // debug_assert!(
                //     !next_saturating_edges_in_T_u.is_empty(),
                //     "No further edges found, but still haven't reached the steiner tree?\n{}",
                //     self.debug_dot_graph()
                // );
                self.flow.saturated_edges.insert(saturating_edge.index());

                // Update all the next saturating edges in T_u
                let v_feeding_terminals = std::mem::take(&mut self.flow.node_to_feeding_terminals[v.index()]);
                let extra_flow_rate = self.flow.node_to_flow_rates[v.index()];
                for edge in next_saturating_edges_in_T_u {
                    let node = edge.target().index();

                    // Algorithm 5
                    self.flow.node_to_feeding_terminals[node].union_with(&v_feeding_terminals);

                    let old_flow_rate = self.flow.node_to_flow_rates[node];
                    let new_flow_rate = old_flow_rate + extra_flow_rate;
                    self.flow.node_to_flow_rates[node] = new_flow_rate;

                    // Algorithm 7
                    if old_flow_rate == 0 {
                        let saturate_time = self.time + (*edge.weight() as Time / extra_flow_rate as Time);
                        self.heap.push(edge.id(), saturate_time.into());
                    } else {
                        let current_saturate_time: Time =
                            (*self.heap.get(&edge.id()).expect("Not in the heap?").1).into();
                        let next_saturate_time = self.time
                            + (current_saturate_time - self.time) * (old_flow_rate as Time / new_flow_rate as Time);
                        self.heap.push_decrease(edge.id(), next_saturate_time.into());
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

        ControlFlow::Continue(())
    }

    /// Traverses saturated subgraph from u, checking for degenerate flow and collecting the next
    /// saturating edges in T_u while we traverse the parents.
    fn detect_generate_flow_and_collect_edges(&mut self, u: NodeIndex, v: NodeIndex) -> DegenerateFlow<'g> {
        #[allow(non_snake_case)]
        let mut next_saturating_edges_in_T_u = Vec::new();

        debug_assert!(self.stack.is_empty());
        self.stack.push(u);
        let new_feeding = &self.state.flow.node_to_feeding_terminals[v.index()];

        while let Some(current) = self.state.stack.pop() {
            // Check for degenerate flow
            let current_feeding = &self.flow.node_to_feeding_terminals[current.index()];
            if !(new_feeding & current_feeding).is_clear() {
                self.stack.clear();
                return DegenerateFlow::Yes; // Degenerate flow detected
            }

            if let Some(edge) = self.find_next_edge_in_T_minus(current) {
                next_saturating_edges_in_T_u.push(edge)
            }

            // Add neighbors reachable through saturated edges
            for edge in self.graph.edges_directed(current, petgraph::Direction::Incoming) {
                if self.flow.saturated_edges[edge.id().index()] {
                    let src = edge.source();
                    self.state.stack.push(src);
                }
            }
        }

        DegenerateFlow::No {
            next_saturating_edges_in_T_u,
        }
    }

    fn debug_dot_graph(&self) -> String {
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
            self.time
        );
        format!(
            "digraph {{\n{:?}{legend}\n}}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel, Config::GraphContentOnly],
                &|_, edge| {
                    let is_in_steiner_tree = self.steiner_tree.edges[edge.id().index()];
                    let is_saturated = self.flow.saturated_edges[edge.id().index()];
                    let is_marked = self.flow.marked_or_saturated_edges[edge.id().index()] && !is_saturated;
                    let attr = match (is_in_steiner_tree, is_saturated, is_marked) {
                        (true, _, _) => "color=forestgreen,fontcolor=forestgreen",
                        (_, true, _) => "color=royalblue,fontcolor=royalblue",
                        (_, false, true) => "color=royalblue,fontcolor=royalblue,style=dashed",
                        (_, _, _) => "",
                    };

                    let mut label = format!("${}", edge.weight());
                    if let Some(suffix) = self.heap.iter().find_map(|(id, priority)| {
                        if *id == edge.id() {
                            Some(format!(" at {}", priority.0))
                        } else {
                            None
                        }
                    }) {
                        label.push_str(&suffix);
                    }
                    Attrs::label(label).with(attr).to_string()
                },
                &|_, (node_ix, _)| {
                    let flow_rate = self.flow.node_to_flow_rates[node_ix.index()];
                    let is_in_steiner_tree = self.steiner_tree.nodes[node_ix.index()];
                    let n = self
                        .graph
                        .edges_directed(node_ix, petgraph::Direction::Incoming)
                        .count();
                    let all_edges_saturated = n > 0
                        && self
                            .graph
                            .edges_directed(node_ix, petgraph::Direction::Incoming)
                            .all(|edge| self.flow.saturated_edges[edge.id().index()]);
                    let all_edges_saturated_or_marked = n > 0
                        && self
                            .graph
                            .edges_directed(node_ix, petgraph::Direction::Incoming)
                            .all(|edge| self.flow.marked_or_saturated_edges[edge.id().index()]);
                    let attr = match (is_in_steiner_tree, all_edges_saturated, all_edges_saturated_or_marked) {
                        (true, _, _) => "color=forestgreen",
                        (_, true, true) => "color=royalblue",
                        (_, false, true) => "color=royalblue,style=dashed",
                        (_, _, _) => "",
                    };
                    Attrs::label(format!(
                        "<{:?} {}&#128167;<br/>{:b}>",
                        &self.graph[node_ix],
                        flow_rate,
                        &self.flow.node_to_feeding_terminals[node_ix.index()],
                    ))
                    .with(attr)
                    .to_string()
                }
            )
        )
    }
}

#[allow(non_snake_case)]
enum DegenerateFlow<'g> {
    Yes,
    No {
        next_saturating_edges_in_T_u: Vec<EdgeReference<'g, Cost>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;

    use petgraph::{Graph, graph::NodeIndex};

    use crate::Cost;

    fn dot_graph(dot: &'static str) -> (Graph<String, Cost>, HashMap<String, NodeIndex>) {
        let dot_graph: dot_parser::canonical::Graph<(&'static str, &'static str)> =
            dot_parser::ast::Graph::try_from(dot).unwrap().into();
        let node_number = dot_graph.nodes.set.len();
        let edge_number = dot_graph.edges.set.len();
        let mut graph = Graph::with_capacity(node_number, edge_number);
        let mut nodes = HashMap::new();
        for (node, attrs) in dot_graph.nodes.set {
            let id = graph.add_node(attrs.id);
            nodes.insert(node, id);
        }
        for edge in dot_graph.edges.set {
            let from_ni = nodes.get(&edge.from).unwrap();
            let to_ni = nodes.get(&edge.to).unwrap();
            let cost = edge
                .attr
                .elems
                .iter()
                .find_map(|(name, value)| {
                    if *name == "cost" {
                        value.parse::<Cost>().ok()
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            graph.add_edge(*from_ni, *to_ni, cost);
        }

        (graph, nodes)
    }

    fn to_steiner_tree_graph(graph: &Graph<String, Cost>, steiner_tree: &SteinerTree) -> String {
        use std::fmt::Write as _;

        let mut out = String::from("digraph {\n");
        let mut steiner_edges = Vec::new();
        for edge in graph.edge_references() {
            if steiner_tree.edges[edge.id().index()] {
                steiner_edges.push(edge);
            }
        }
        steiner_edges.sort_by_key(|edge| (&graph[edge.source()], &graph[edge.target()]));
        for edge in steiner_edges {
            writeln!(&mut out, "  {} -> {}", &graph[edge.source()], &graph[edge.target()]).unwrap();
        }
        out.push('}');
        out
    }

    struct Runner {
        graph: Graph<String, Cost>,
        nodes: HashMap<String, NodeIndex>,
        greedy_flac: GreedyFlac,
        steiner_tree: SteinerTree,
    }

    impl Runner {
        fn from_dot_graph(dot: &'static str) -> Self {
            let (graph, nodes) = dot_graph(dot);
            let steiner_tree = SteinerTree::new(&graph, nodes["root"]);
            // Collect terminals: nodes starting with 't' (t1, t2, terminal, etc.) but not "top"
            let terminals = nodes
                .iter()
                .filter_map(|(k, v)| {
                    let mut chars = k.chars();
                    if chars.next() == Some('t') && chars.all(|c| c.is_ascii_digit()) {
                        Some(*v)
                    } else {
                        None
                    }
                })
                .collect();
            let greedy_flac = GreedyFlac::new(&graph, terminals);
            Self {
                graph,
                nodes,
                greedy_flac,
                steiner_tree,
            }
        }

        fn run_once(&mut self) -> ControlFlow<()> {
            self.greedy_flac.run_once(&self.graph, &mut self.steiner_tree)
        }

        fn run(&mut self) -> Cost {
            self.greedy_flac.run(&self.graph, &mut self.steiner_tree);
            self.steiner_tree.total_weight
        }

        fn debug_graph(&mut self) -> String {
            Flac {
                state: &mut self.greedy_flac,
                graph: &self.graph,
                steiner_tree: &mut self.steiner_tree,
            }
            .debug_dot_graph()
        }

        fn steiner_graph(&self) -> String {
            to_steiner_tree_graph(&self.graph, &self.steiner_tree)
        }

        fn extend_terminals(&mut self, terminal_names: &[&str]) {
            let terminals: Vec<NodeIndex> = terminal_names.iter().map(|name| self.nodes[*name]).collect();
            self.greedy_flac.extend_terminals(terminals);
        }
    }

    #[test]
    fn step_by_step() {
        let mut runner = Runner::from_dot_graph(
            r#"digraph {
            root -> a;
            a -> t1 [cost=1];
            a -> t2 [cost=2];
            }"#,
        );

        let outcome = runner.run_once();
        assert!(outcome.is_continue(), "\n{}", runner.debug_graph());
        insta::assert_snapshot!(runner.steiner_graph(), @r"
        digraph {
          a -> t1
          root -> a
        }
        ");

        let outcome = runner.run_once();
        assert!(outcome.is_break(), "\n{}", runner.debug_graph());
        insta::assert_snapshot!(runner.steiner_graph(), @r"
        digraph {
          a -> t1
          a -> t2
          root -> a
        }
        ");
    }

    #[test]
    fn single_terminal_direct_path() {
        let mut runner = Runner::from_dot_graph(
            r#"digraph {
            root -> t1 [cost=5];
            }"#,
        );

        let total_cost = runner.run();
        assert_eq!(total_cost, 5, "\n{}", runner.debug_graph());
        insta::assert_snapshot!(runner.steiner_graph(), @r"
        digraph {
          root -> t1
        }
        ");
    }

    #[test]
    fn multiple_terminals_shared_edges() {
        let mut runner = Runner::from_dot_graph(
            r#"digraph {
            root -> shared [cost=10];
            shared -> t1 [cost=3];
            shared -> t2 [cost=5];
            shared -> t3 [cost=2];
            }"#,
        );

        let total_cost = runner.run();
        assert_eq!(
            total_cost,
            20, // 10 + 3 + 5 + 2
            "\n{}",
            runner.debug_graph()
        );
        insta::assert_snapshot!(runner.steiner_graph(), @r"
        digraph {
          root -> shared
          shared -> t1
          shared -> t2
          shared -> t3
        }
        ");
    }

    #[test]
    fn degenerate_flow_detection() {
        // Graph where t1 can reach a through two paths, which would create degenerate flow
        let mut runner = Runner::from_dot_graph(
            r#"digraph {
            root -> a [cost=10];
            a -> b [cost=2];
            a -> c [cost=3];
            b -> t1 [cost=1];
            c -> t1 [cost=1];
            }"#,
        );

        let total_cost = runner.run();
        assert_eq!(
            total_cost,
            13, // Should pick one path: root -> a -> b -> t1 (10 + 2 + 1)
            "\n{}",
            runner.debug_graph()
        );
        // The algorithm should mark one edge to avoid degenerate flow
        insta::assert_snapshot!(runner.steiner_graph(), @r"
        digraph {
          a -> b
          b -> t1
          root -> a
        }
        ");
    }

    #[test]
    fn complex_graph_multiple_paths() {
        let mut runner = Runner::from_dot_graph(
            r#"digraph {
            root -> a [cost=5];
            root -> b [cost=8];
            a -> c [cost=3];
            b -> c [cost=2];
            c -> t1 [cost=4];
            c -> t2 [cost=6];
            a -> t2 [cost=10];
            b -> t3 [cost=7];
            }"#,
        );

        let total_cost = runner.run();
        assert_eq!(
            total_cost,
            // root -> a -> c -> t1,t2: 5 + 3 + 6 + 4 = 18
            // root -> b -> t3: 8 + 7 = 15
            33,
            "\n{}",
            runner.debug_graph()
        );
        insta::assert_snapshot!(runner.steiner_graph(), @r"
        digraph {
          a -> c
          b -> t3
          c -> t1
          c -> t2
          root -> a
          root -> b
        }
        ");
    }

    #[test]
    fn incremental_terminal_addition() {
        // Start with only t1 terminal by creating custom runner
        let (graph, nodes) = dot_graph(
            r#"digraph {
            root -> a [cost=4];
            root -> b [cost=6];
            a -> t1 [cost=2];
            b -> t2 [cost=3];
            a -> t3 [cost=5];
            }"#,
        );
        let steiner_tree = SteinerTree::new(&graph, nodes["root"]);
        let greedy_flac = GreedyFlac::new(&graph, vec![nodes["t1"]]);
        let mut runner = Runner {
            graph,
            nodes,
            greedy_flac,
            steiner_tree,
        };

        // First run with only t1
        let total_cost = runner.run();
        assert_eq!(
            total_cost,
            6, // root -> a -> t1: 4 + 2
            "\n{}",
            runner.debug_graph()
        );

        // Add t2 as a new terminal
        runner.extend_terminals(&["t2"]);
        let total_cost = runner.run();
        assert_eq!(
            total_cost,
            6 + 9, // root -> b -> t2: 6 + 3
            "\n{}",
            runner.debug_graph()
        );

        // Add t3 as another terminal
        runner.extend_terminals(&["t3"]);
        let total_cost = runner.run();
        assert_eq!(
            total_cost,
            6 + 9 + 5, // a -> t3: 5 (root -> a already in tree)
            "\n{}",
            runner.debug_graph()
        );

        insta::assert_snapshot!(runner.steiner_graph(), @r"
        digraph {
          a -> t1
          a -> t3
          b -> t2
          root -> a
          root -> b
        }
        ");
    }

    #[test]
    fn weighted_edges_different_costs() {
        let mut runner = Runner::from_dot_graph(
            r#"digraph {
            root -> a [cost=1];
            root -> b [cost=100];
            a -> c [cost=50];
            b -> c [cost=1];
            c -> t1 [cost=1];
            a -> t2 [cost=2];
            b -> t3 [cost=2];
            }"#,
        );

        let total_cost = runner.run();
        assert_eq!(
            total_cost,
            // Optimal: root->a->t2 (3), root->b->c->t1 (102), root->b->t3 (102 already counted + 2 = 2)
            // But GreedyFLAC doesn't take the optimal path.
            156,
            "\n{}",
            runner.debug_graph()
        );

        // The algorithm should prefer the cheaper paths
        insta::assert_snapshot!(runner.steiner_graph(), @r"
        digraph {
          a -> c
          a -> t2
          b -> t3
          c -> t1
          root -> a
          root -> b
        }
        ");
    }

    #[test]
    fn diamond_shaped_graph() {
        let mut runner = Runner::from_dot_graph(
            r#"digraph {
            root -> top [cost=5];
            top -> left [cost=3];
            top -> right [cost=4];
            left -> bottom [cost=6];
            right -> bottom [cost=2];
            bottom -> t1 [cost=1];
            left -> t2 [cost=8];
            right -> t3 [cost=7];
            }"#,
        );

        let total_cost = runner.run();
        assert_eq!(
            total_cost,
            30, // root->top->left->t2 (5+3+8=16) + top->right->bottom->t1 (4+2+1=7) + right->t3 (7-already have right)
            "\n{}",
            runner.debug_graph()
        );

        insta::assert_snapshot!(runner.steiner_graph(), @r"
        digraph {
          bottom -> t1
          left -> t2
          right -> bottom
          right -> t3
          root -> top
          top -> left
          top -> right
        }
        ");
    }

    #[test]
    fn linear_chain_graph() {
        let mut runner = Runner::from_dot_graph(
            r#"digraph {
            root -> n1 [cost=2];
            n1 -> n2 [cost=3];
            n2 -> n3 [cost=4];
            n3 -> n4 [cost=5];
            n4 -> t1 [cost=6];
            n2 -> t2 [cost=10];
            n3 -> t3 [cost=8];
            }"#,
        );

        let total_cost = runner.run();
        assert_eq!(
            total_cost,
            38, // root->n1->n2->n3->n4->t1 (2+3+4+5+6=20) + n2->t2 (10) + n3->t3 (8)
            "\n{}",
            runner.debug_graph()
        );

        insta::assert_snapshot!(runner.steiner_graph(), @r"
        digraph {
          n1 -> n2
          n2 -> n3
          n2 -> t2
          n3 -> n4
          n3 -> t3
          n4 -> t1
          root -> n1
        }
        ");
    }
}
