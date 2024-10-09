use std::path::Path;

use itertools::Itertools;
use petgraph::{graph::NodeIndex, stable_graph::StableGraph};

use crate::Cost;

type SteinLibGraph = StableGraph<(), Cost>;

/// https://steinlib.zib.de/showset.php?GENE
/// Directed acyclic graphs
pub(super) struct GeneGraph {
    pub name: &'static str,
    pub optimal_cost: Cost,
    pub graph: SteinLibGraph,
    pub root: NodeIndex,
    pub terminals: Vec<NodeIndex>,
}

pub(super) fn load_dataset() -> impl Iterator<Item = GeneGraph> {
    let cases = vec![
        ("gene42.stp", 126),
        ("gene61a.stp", 205),
        ("gene61b.stp", 199),
        ("gene61c.stp", 196),
        ("gene61f.stp", 198),
    ];

    cases.into_iter().map(|(name, cost)| load(name, cost))
}

fn load(name: &'static str, optimal_cost: Cost) -> GeneGraph {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/steiner_tree/tests/steinlib/GENE")
        .join(name);

    let content = std::fs::read_to_string(path).unwrap();

    let sections = content.split("SECTION").collect::<Vec<_>>();
    let mut graph_section = sections[2].split('\n');
    graph_section.next().unwrap(); // " Graph" after SECTION.

    fn parse_value(key: &'static str, line: &str) -> usize {
        let (k, value) = line.split_whitespace().collect_tuple().unwrap();
        assert_eq!(k, key);
        value.parse().unwrap()
    }

    let node_count = parse_value("Nodes", graph_section.next().unwrap());
    let edge_count = parse_value("Edges", graph_section.next().unwrap());

    let mut graph = SteinLibGraph::new();

    for _ in 0..node_count {
        graph.add_node(());
    }

    for line in graph_section {
        let Some((_, source, target, weight, _)) = line.split_whitespace().collect_tuple() else {
            break;
        };
        let source: usize = source.parse().unwrap();
        let target: usize = target.parse().unwrap();
        let weight: Cost = weight.parse().unwrap();
        graph.add_edge(NodeIndex::new(source - 1), NodeIndex::new(target - 1), weight);
    }

    assert_eq!(graph.node_count(), node_count);
    assert_eq!(graph.edge_count(), edge_count);
    assert!(!petgraph::algo::is_cyclic_directed(&graph));

    let mut terminals_section = sections[3].split('\n');
    terminals_section.next().unwrap(); // " Terminals" after SECTION.

    let terminals_count = parse_value("Terminals", terminals_section.next().unwrap());
    let root = NodeIndex::new(parse_value("Root", terminals_section.next().unwrap()) - 1);

    let mut terminals = Vec::<NodeIndex>::new();
    for line in terminals_section {
        let Some((_, terminal)) = line.split_whitespace().collect_tuple() else {
            break;
        };
        let terminal: usize = terminal.parse().unwrap();
        let terminal = NodeIndex::new(terminal - 1);
        if root != terminal {
            terminals.push(terminal);
        }
    }

    assert_eq!(terminals.len() + 1, terminals_count);

    terminals.sort_unstable();

    GeneGraph {
        name,
        optimal_cost,
        graph,
        root,
        terminals,
    }
}
