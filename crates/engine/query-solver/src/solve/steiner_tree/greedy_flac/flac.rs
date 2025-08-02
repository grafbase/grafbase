use crate::{Cost, dot_graph::Attrs};
use fixedbitset::FixedBitSet;
use itertools::Itertools as _;
use petgraph::{
    Graph,
    graph::{EdgeIndex, EdgeReference, NodeIndex},
    visit::{EdgeRef, NodeIndexable},
};
use priority_queue::PriorityQueue;
use std::{cmp::Ordering, ops::ControlFlow};

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

pub(super) struct Flac {
    // Graph state
    pub steiner_tree_nodes: FixedBitSet,
    pub steiner_tree_edges: FixedBitSet,
    pub weights: Vec<Cost>,
    pub total_cost: Cost,
    // Algorithm state
    saturated_edges: FixedBitSet,
    marked_or_saturated_edges: FixedBitSet,
    root_feeding_terminals: FixedBitSet,
    node_to_feeding_terminals: Vec<FixedBitSet>,
    node_to_flow_rates: Vec<FlowRate>,
    terminals: Vec<NodeIndex>,
    // Run state, re-used across each run
    time: Time,
    heap: PriorityQueue<EdgeIndex, Priority>,
    stack: Vec<NodeIndex>,
}

impl Flac {
    pub fn new<N>(graph: &Graph<N, Cost>, terminals: Vec<NodeIndex>, steiner_tree_nodes: FixedBitSet) -> Self {
        debug_assert!(!steiner_tree_nodes.is_empty(), "Root must be part of the steiner tree.");
        Self {
            time: 0.0,
            heap: PriorityQueue::new(),
            steiner_tree_nodes,
            steiner_tree_edges: FixedBitSet::with_capacity(graph.edge_count()),
            total_cost: 0,
            saturated_edges: FixedBitSet::with_capacity(graph.edge_count()),
            marked_or_saturated_edges: FixedBitSet::with_capacity(graph.edge_count()),
            root_feeding_terminals: FixedBitSet::new(),
            node_to_feeding_terminals: vec![FixedBitSet::new(); graph.node_bound()],
            node_to_flow_rates: vec![0; graph.node_bound()],
            terminals,
            weights: graph.edge_weights().cloned().collect(),
            stack: Vec::new(),
        }
    }

    pub fn extend_terminals(&mut self, terminals: impl IntoIterator<Item = NodeIndex>) {
        self.terminals.extend(terminals);
    }

    pub fn run<N>(&mut self, graph: &Graph<N, Cost>) -> Outcome
    where
        N: std::fmt::Debug,
    {
        Runner { state: self, graph }.run()
    }

    pub fn greedy_run<N>(&mut self, graph: &Graph<N, Cost>) -> Cost
    where
        N: std::fmt::Debug,
    {
        let mut runner = Runner { state: self, graph };
        loop {
            println!("\n\n=== NEW RUN ===\n\n{}", runner.debug_dot_graph());
            let Outcome { is_finished } = runner.run();
            println!("{}", runner.debug_dot_graph());
            if is_finished {
                return self.total_cost;
            }
        }
    }

    pub fn reset(&mut self) {
        self.total_cost = 0;
        self.terminals.clear();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Outcome {
    pub is_finished: bool,
}

struct Runner<'s, 'g, N> {
    state: &'s mut Flac,
    graph: &'g Graph<N, Cost>,
}

impl<N> std::ops::Deref for Runner<'_, '_, N> {
    type Target = Flac;
    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<N> std::ops::DerefMut for Runner<'_, '_, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state
    }
}

impl<'g, N> Runner<'_, 'g, N>
where
    N: std::fmt::Debug,
{
    fn run(&mut self) -> Outcome {
        // Prepare the initial state
        debug_assert!(self.stack.is_empty());
        self.time = 0.0;
        self.heap.clear();
        self.saturated_edges.clear();
        self.marked_or_saturated_edges.clear();
        self.node_to_feeding_terminals.iter_mut().for_each(|set| set.clear());
        self.node_to_flow_rates.fill(0);

        // Initialize the state with the current terminals. New ones may have been added since the
        // last run.
        let n_terminals = self.state.terminals.len();
        self.state.root_feeding_terminals.grow(n_terminals);
        for ix in self.state.root_feeding_terminals.zeroes() {
            let terminal = self.state.terminals[ix];
            if let Some(edge) = self.find_next_edge_in_T_minus(terminal) {
                let saturate_time = self.time + *edge.weight() as Time;
                self.state.heap.push(edge.id(), saturate_time.into());
                let feeding = &mut self.state.node_to_feeding_terminals[terminal.index()];
                feeding.grow(n_terminals);
                feeding.insert(ix);
                self.state.node_to_flow_rates[terminal.index()] = 1;
            }
        }

        // Run the algorithm
        loop {
            let Some(edge) = self.get_next_saturating_edge() else {
                unreachable!("Could not reach root?\n{}", self.debug_dot_graph());
            };

            // The new update_flow_rates handles degenerate flow checking internally
            if let ControlFlow::Break((_, v)) = self.update_flow_rates(edge) {
                println!("{}", self.debug_dot_graph());
                let new_feeding_terminals = &self.state.node_to_feeding_terminals[v.index()];
                debug_assert!(
                    !new_feeding_terminals.is_clear(),
                    "No new terminals?\n{}",
                    self.debug_dot_graph()
                );
                debug_assert!(
                    (new_feeding_terminals & (&self.root_feeding_terminals)).is_clear(),
                    "New feeding terminals weren't distinct from the current ones\n{}\n{:b}\n{:b}\n{}",
                    self.terminals
                        .iter()
                        .map(|idx| &self.graph[*idx])
                        .format_with(",", |node, f| f(&format_args!("{node:?}"))),
                    &new_feeding_terminals,
                    &self.root_feeding_terminals,
                    self.debug_dot_graph()
                );

                self.state.root_feeding_terminals.union_with(new_feeding_terminals);
                let is_finished = self.root_feeding_terminals.is_full();

                self.total_cost += self.graph[edge];
                self.steiner_tree_edges.insert(edge.index());

                debug_assert!(self.stack.is_empty());
                self.stack.push(v);
                while let Some(node) = self.stack.pop() {
                    self.steiner_tree_nodes.insert(node.index());
                    for edge in self.graph.edges_directed(node, petgraph::Direction::Outgoing) {
                        if self.saturated_edges[edge.id().index()] {
                            self.steiner_tree_edges.insert(edge.id().index());
                            self.total_cost += *edge.weight();
                            self.stack.push(edge.target());
                        }
                    }
                }

                return Outcome { is_finished };
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
            if !self.marked_or_saturated_edges.contains(edge.id().index()) {
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
        self.marked_or_saturated_edges.insert(saturating_edge.index());

        // When the algorithm reaches a node of the Steiner Tree, which starts with only the root node,
        // we don't need to go further.
        if self.steiner_tree_nodes[u.index()] {
            return ControlFlow::Break((u, v));
        }

        // Algorithm 9
        // Check if flow would be degenerate and collect edges to update
        match self.detect_generate_flow_and_collect_edges(u, v) {
            DegenerateFlow::Yes => {}
            DegenerateFlow::No {
                next_saturating_edges_in_T_u,
            } => {
                debug_assert!(
                    !next_saturating_edges_in_T_u.is_empty(),
                    "No further edges found, but still haven't reached the steiner tree?\n{}",
                    self.debug_dot_graph()
                );
                self.saturated_edges.insert(saturating_edge.index());

                // Update all the next saturating edges in T_u
                let v_feeding_terminals = std::mem::take(&mut self.node_to_feeding_terminals[v.index()]);
                let extra_flow_rate = v_feeding_terminals.count_ones(..) as FlowRate;
                for edge in next_saturating_edges_in_T_u {
                    let node = edge.target().index();

                    // Algorithm 5
                    self.node_to_feeding_terminals[node].union_with(&v_feeding_terminals);

                    let old_flow_rate = self.node_to_flow_rates[node];
                    let new_flow_rate = old_flow_rate + extra_flow_rate;
                    self.node_to_flow_rates[node] = new_flow_rate;

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
                self.node_to_feeding_terminals[v.index()] = v_feeding_terminals;
            }
        }

        // Algorithm 8
        if let Some(edge) = self.find_next_edge_in_T_minus(v) {
            let flow_rate = self.node_to_flow_rates[v.index()];
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
        let new_feeding = &self.state.node_to_feeding_terminals[v.index()];

        while let Some(current) = self.state.stack.pop() {
            // Check for degenerate flow
            let current_feeding = &self.node_to_feeding_terminals[current.index()];
            if !(new_feeding & current_feeding).is_clear() {
                return DegenerateFlow::Yes; // Degenerate flow detected
            }

            if let Some(edge) = self.find_next_edge_in_T_minus(current) {
                next_saturating_edges_in_T_u.push(edge)
            }

            // Add neighbors reachable through saturated edges
            for edge in self.graph.edges_directed(current, petgraph::Direction::Incoming) {
                if self.saturated_edges[edge.id().index()] {
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
                    let is_in_steiner_tree = self.steiner_tree_edges[edge.id().index()];
                    let is_saturated = self.saturated_edges[edge.id().index()];
                    let is_marked = self.marked_or_saturated_edges[edge.id().index()] && !is_saturated;
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
                    let flow_rate = self.node_to_flow_rates[node_ix.index()];
                    let is_in_steiner_tree = self.steiner_tree_nodes[node_ix.index()];
                    let n = self
                        .graph
                        .edges_directed(node_ix, petgraph::Direction::Incoming)
                        .count();
                    let all_edges_saturated = n > 0
                        && self
                            .graph
                            .edges_directed(node_ix, petgraph::Direction::Incoming)
                            .all(|edge| self.saturated_edges[edge.id().index()]);
                    let all_edges_saturated_or_marked = n > 0
                        && self
                            .graph
                            .edges_directed(node_ix, petgraph::Direction::Incoming)
                            .all(|edge| self.marked_or_saturated_edges[edge.id().index()]);
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
                        &self.node_to_feeding_terminals[node_ix.index()],
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

    use petgraph::{Graph, dot::dot_parser::ParseFromDot, graph::NodeIndex};

    use crate::Cost;

    fn dot_graph(dot: &'static str) -> (Graph<String, Cost>, HashMap<String, NodeIndex>) {
        let g: Graph<_, _> = ParseFromDot::from_dot_graph(dot_parser::ast::Graph::try_from(dot).unwrap());
        let mut nodes = HashMap::new();
        let steiner_graph = g.map(
            |idx, node| {
                nodes.insert(node.id.to_string(), idx);
                node.id.to_string()
            },
            |_, attrs| {
                attrs
                    .elems
                    .iter()
                    .find_map(|(name, value)| {
                        if *name == "cost" {
                            value.parse::<Cost>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default()
            },
        );
        (steiner_graph, nodes)
    }

    fn to_dot_graph(graph: &Graph<String, Cost>, flac: &Flac) -> String {
        use std::fmt::Write as _;

        let mut out = String::from("digraph {\n");
        let mut steiner_edges = Vec::new();
        for edge in graph.edge_references() {
            if flac.steiner_tree_edges[edge.id().index()] {
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

    fn build<'a>(
        graph: &Graph<String, Cost>,
        nodes: &HashMap<String, NodeIndex>,
        root: &'a str,
        terminals: impl IntoIterator<Item = &'a str>,
    ) -> Flac {
        let mut steiner_tree_nodes = FixedBitSet::with_capacity(graph.node_bound());
        steiner_tree_nodes.insert(nodes[root].index());
        Flac::new(
            graph,
            terminals.into_iter().map(|label| nodes[label]).collect(),
            steiner_tree_nodes,
        )
    }

    fn debug_graph(graph: &Graph<String, Cost>, flac: &mut Flac) -> String {
        Runner { state: flac, graph }.debug_dot_graph()
    }

    #[test]
    fn step_by_step() {
        let (graph, nodes) = dot_graph(
            r#"digraph {
            root -> a;
            a -> b [cost=1];
            a -> c [cost=2];
            }"#,
        );
        let mut flac = build(&graph, &nodes, "root", ["b", "c"]);

        let outcome = flac.run(&graph);
        assert_eq!(
            outcome,
            Outcome { is_finished: false },
            "\n{}",
            debug_graph(&graph, &mut flac)
        );
        insta::assert_snapshot!(to_dot_graph(&graph, &flac), @r"
        digraph {
          a -> b
          root -> a
        }
        ");

        let outcome = flac.run(&graph);
        assert_eq!(
            outcome,
            Outcome { is_finished: true },
            "\n{}",
            debug_graph(&graph, &mut flac)
        );
        insta::assert_snapshot!(to_dot_graph(&graph, &flac), @r"
        digraph {
          a -> b
          a -> c
          root -> a
        }
        ");
    }

    #[test]
    fn single_terminal_direct_path() {
        let (graph, nodes) = dot_graph(
            r#"digraph {
            root -> terminal [cost=5];
            }"#,
        );
        let mut flac = build(&graph, &nodes, "root", ["terminal"]);

        let total_cost = flac.greedy_run(&graph);
        assert_eq!(total_cost, 5, "\n{}", debug_graph(&graph, &mut flac));
        insta::assert_snapshot!(to_dot_graph(&graph, &flac), @r"
        digraph {
          root -> terminal
        }
        ");
    }

    #[test]
    fn multiple_terminals_shared_edges() {
        let (graph, nodes) = dot_graph(
            r#"digraph {
            root -> shared [cost=10];
            shared -> t1 [cost=3];
            shared -> t2 [cost=5];
            shared -> t3 [cost=2];
            }"#,
        );
        let mut flac = build(&graph, &nodes, "root", ["t1", "t2", "t3"]);

        let total_cost = flac.greedy_run(&graph);
        assert_eq!(
            total_cost,
            20, // 10 + 3 + 5 + 2
            "\n{}",
            debug_graph(&graph, &mut flac)
        );
        insta::assert_snapshot!(to_dot_graph(&graph, &flac), @r"
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
        let (graph, nodes) = dot_graph(
            r#"digraph {
            root -> a [cost=10];
            a -> b [cost=2];
            a -> c [cost=3];
            b -> t1 [cost=1];
            c -> t1 [cost=1];
            }"#,
        );
        let mut flac = build(&graph, &nodes, "root", ["t1"]);

        let total_cost = flac.greedy_run(&graph);
        assert_eq!(
            total_cost,
            13, // Should pick one path: root -> a -> b -> t1 (10 + 2 + 1)
            "\n{}",
            debug_graph(&graph, &mut flac)
        );
        // The algorithm should mark one edge to avoid degenerate flow
        insta::assert_snapshot!(to_dot_graph(&graph, &flac), @r"
        digraph {
          a -> b
          b -> t1
          root -> a
        }
        ");
    }

    #[test]
    fn complex_graph_multiple_paths() {
        let (graph, nodes) = dot_graph(
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
        let mut flac = build(&graph, &nodes, "root", ["t1", "t2", "t3"]);

        let total_cost = flac.greedy_run(&graph);
        assert_eq!(
            total_cost,
            // root -> a -> c -> t1,t2: 5 + 3 + 6 + 4 = 18
            // root -> b -> t3: 8 + 7 = 15
            33,
            "\n{}",
            debug_graph(&graph, &mut flac)
        );
        insta::assert_snapshot!(to_dot_graph(&graph, &flac), @r"
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
        let (graph, nodes) = dot_graph(
            r#"digraph {
            root -> a [cost=4];
            root -> b [cost=6];
            a -> t1 [cost=2];
            b -> t2 [cost=3];
            a -> t3 [cost=5];
            }"#,
        );
        let mut flac = build(&graph, &nodes, "root", ["t1"]);

        // First run with only t1
        let total_cost = flac.greedy_run(&graph);
        assert_eq!(
            total_cost,
            6, // root -> a -> t1: 4 + 2
            "\n{}",
            debug_graph(&graph, &mut flac)
        );

        // Add t2 as a new terminal
        flac.extend_terminals(vec![nodes["t2"]]);
        let total_cost = flac.greedy_run(&graph);
        assert_eq!(
            total_cost,
            6 + 9, // root -> b -> t2: 6 + 3
            "\n{}",
            debug_graph(&graph, &mut flac)
        );

        // Add t3 as another terminal
        flac.extend_terminals(vec![nodes["t3"]]);
        let total_cost = flac.greedy_run(&graph);
        assert_eq!(
            total_cost,
            6 + 9 + 5, // a -> t3: 5 (root -> a already in tree)
            "\n{}",
            debug_graph(&graph, &mut flac)
        );

        insta::assert_snapshot!(to_dot_graph(&graph, &flac), @r"
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
        let (graph, nodes) = dot_graph(
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
        let mut flac = build(&graph, &nodes, "root", ["t1", "t2", "t3"]);

        let total_cost = flac.greedy_run(&graph);
        assert_eq!(
            total_cost,
            105, // Optimal: root->a->t2 (3), root->b->c->t1 (102), root->b->t3 (102 already counted + 2 = 2)
            "\n{}",
            debug_graph(&graph, &mut flac)
        );

        // The algorithm should prefer the cheaper paths
        insta::assert_snapshot!(to_dot_graph(&graph, &flac), @r"
        digraph {
          root -> a
          a -> t2
          root -> b
          b -> c
          c -> t1
          b -> t3
        }
        ");
    }

    #[test]
    fn diamond_shaped_graph() {
        let (graph, nodes) = dot_graph(
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
        let mut flac = build(&graph, &nodes, "root", ["t1", "t2", "t3"]);

        let total_cost = flac.greedy_run(&graph);
        assert_eq!(
            total_cost,
            30, // root->top->left->t2 (5+3+8=16) + top->right->bottom->t1 (4+2+1=7) + right->t3 (7-already have right)
            "\n{}",
            debug_graph(&graph, &mut flac)
        );

        insta::assert_snapshot!(to_dot_graph(&graph, &flac), @r"
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
        let (graph, nodes) = dot_graph(
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
        let mut flac = build(&graph, &nodes, "root", ["t1", "t2", "t3"]);

        let total_cost = flac.greedy_run(&graph);
        assert_eq!(
            total_cost,
            38, // root->n1->n2->n3->n4->t1 (2+3+4+5+6=20) + n2->t2 (10) + n3->t3 (8)
            "\n{}",
            debug_graph(&graph, &mut flac)
        );

        insta::assert_snapshot!(to_dot_graph(&graph, &flac), @r"
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
