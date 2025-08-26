mod cases;
mod gene;

use std::collections::HashMap;

use petgraph::{Graph, graph::NodeIndex, visit::EdgeRef as _};

use crate::solve::{
    input::SteinerWeight,
    steiner_tree::{GreedyFlac, SteinerTree},
};

struct Runner {
    graph: Graph<String, SteinerWeight>,
    nodes: HashMap<String, NodeIndex>,
    greedy_flac: GreedyFlac,
    steiner_tree: SteinerTree,
}

impl Runner {
    fn from_dot_graph(dot: &'static str) -> Self {
        let (graph, nodes) = dot_graph(dot);
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
        let steiner_tree = SteinerTree::new(&graph, nodes["root"], terminals);
        let greedy_flac = GreedyFlac::new(&graph);
        Self {
            graph,
            nodes,
            greedy_flac,
            steiner_tree,
        }
    }

    fn run(&mut self) -> SteinerWeight {
        self.greedy_flac.run(&self.graph, &mut self.steiner_tree);
        self.steiner_tree.total_weight
    }

    fn debug_graph(&self) -> String {
        self.greedy_flac.debug_dot_graph(&self.graph, &self.steiner_tree)
    }

    fn steiner_graph(&self) -> String {
        to_steiner_tree_graph(&self.graph, &self.steiner_tree)
    }

    fn extend_terminals(&mut self, terminal_names: &[&str]) {
        let _ = self
            .steiner_tree
            .extend_terminals(terminal_names.iter().map(|name| self.nodes[*name]));
    }
}

fn dot_graph(dot: &'static str) -> (Graph<String, SteinerWeight>, HashMap<String, NodeIndex>) {
    let ast_graph = dot_parser::ast::Graph::try_from(dot).unwrap();
    let canonical_graph: dot_parser::canonical::Graph<(String, String)> =
        dot_parser::canonical::Graph::from(ast_graph).filter_map(|(name, value)| Some((name.into(), value.into())));

    let node_number = canonical_graph.nodes.set.len();
    let edge_number = canonical_graph.edges.set.len();
    let mut graph = Graph::with_capacity(node_number, edge_number);
    let mut nodes = HashMap::new();
    for (node, attrs) in canonical_graph.nodes.set {
        let id = graph.add_node(attrs.id);
        nodes.insert(node, id);
    }
    for edge in canonical_graph.edges.set {
        let from_ni = nodes.get(&edge.from).unwrap();
        let to_ni = nodes.get(&edge.to).unwrap();
        let weight = edge
            .attr
            .elems
            .iter()
            .find_map(|(name, value)| {
                if name == "label" {
                    value.parse::<SteinerWeight>().ok()
                } else {
                    None
                }
            })
            .unwrap_or_default();
        graph.add_edge(*from_ni, *to_ni, weight);
    }

    (graph, nodes)
}

fn to_steiner_tree_graph(graph: &Graph<String, SteinerWeight>, steiner_tree: &SteinerTree) -> String {
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
