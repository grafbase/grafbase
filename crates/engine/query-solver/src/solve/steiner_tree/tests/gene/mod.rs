mod loader;
mod report;

use std::time::{Duration, Instant};

use petgraph::{prelude::StableGraph, visit::IntoNodeReferences};

use crate::solve::steiner_tree::{GreedyFlac, SteinerTree};
use report::*;

/// Sanity check our GreedyFlac Steiner Tree algorithm against a graph with known optimal weight.
#[test]
fn steinlib_gene() {
    let mut reports = Vec::new();

    for gene in loader::load_gene_dataset() {
        let start = Instant::now();

        let mut steiner_tree = SteinerTree::new(&gene.graph, gene.root, gene.terminals.clone());
        let mut greddy_flac = GreedyFlac::new(&gene.graph);
        let prepare_duration = start.elapsed();

        let start = Instant::now();
        greddy_flac.run(&gene.graph, &mut steiner_tree);
        let total_weight = steiner_tree.total_weight;
        let grow_duration = start.elapsed();

        let steiner_tree_node_count = gene
            .graph
            .node_references()
            .filter(|(node_id, _)| steiner_tree.nodes[node_id.index()])
            .count();

        assert!(
            gene.terminals
                .iter()
                .all(|terminal| steiner_tree.nodes[terminal.index()])
        );

        // Are all the terminals accessible from root?
        let mut graph = StableGraph::from(gene.graph.clone());
        graph.retain_nodes(|_, node| steiner_tree.nodes[node.index()]);
        for terminal in &gene.terminals {
            assert!(petgraph::algo::has_path_connecting(&graph, gene.root, *terminal, None));
        }

        reports.push(AlgorithmRunReport {
            name: gene.name,
            algorithm: "GreedyFlac",
            weight: total_weight,
            optimal_weight: gene.optimal_weight,
            node_count: steiner_tree_node_count,
            kept_nodes_percentage: ((steiner_tree_node_count * 100) as f64) / (gene.graph.node_count() as f64),
            prepare_duration,
            grow_duration,
        });
    }

    let report = TestReport {
        algorithm: "GreedyFlac",
        reports,
    };
    println!("{report}");

    for result in &report.reports {
        assert!(
            (result.weight as f64 / result.optimal_weight as f64) <= 1.05,
            "Weight difference is too big for {}",
            result.name,
        );
        assert!(
            result.prepare_duration + result.grow_duration < Duration::from_millis(200),
            "Total time is too long for {}",
            result.name,
        );
    }
}
