mod loader;
mod report;

use std::time::{Duration, Instant};

use petgraph::visit::{EdgeRef, IntoNodeReferences};

use crate::solve::steiner_tree::{GreedyFlac, SteinerContext, SteinerTree};
use report::*;

/// Sanity check our GreedyFlac Steiner Tree algorithm against a graph with known optimal cost.
#[test]
fn greedy_flac_steinlib_gene() {
    let mut all_results = Vec::new();

    for gene in loader::load_gene_dataset() {
        let start = Instant::now();
        let ctx = SteinerContext::build(
            &gene.graph,
            gene.root,
            |(node, _)| Some(node),
            |edge| Some((edge.id(), edge.source(), edge.target(), *edge.weight())),
        );

        let terminals = gene
            .terminals
            .iter()
            .map(|node| ctx.to_node_ix(*node))
            .collect::<Vec<_>>();

        let mut steiner_tree = SteinerTree::new(&ctx.graph, ctx.root_ix);
        let mut flac = GreedyFlac::new(&ctx.graph, terminals);
        let prepare_duration = start.elapsed();

        let start = Instant::now();
        flac.run(&ctx.graph, &mut steiner_tree);
        let total_cost = steiner_tree.total_weight;
        let grow_duration = start.elapsed();

        let steiner_tree_node_count = gene
            .graph
            .node_references()
            .filter(|(node_id, _)| steiner_tree.nodes[ctx.to_node_ix(*node_id).index()])
            .count();

        let ratio = (total_cost as f64) / (gene.optimal_cost as f64);

        let main_result = AlgorithmResult {
            cost: total_cost,
            optimal_cost: gene.optimal_cost,
            node_count: steiner_tree_node_count,
            kept_nodes_percentage: ((steiner_tree_node_count * 100) as f64) / (gene.graph.node_count() as f64),
            prepare_duration,
            grow_duration,
        };

        assert!(
            gene.terminals
                .iter()
                .all(|terminal| steiner_tree.nodes[ctx.to_node_ix(*terminal).index()])
        );

        // Are all the terminals accessible from root?
        let mut graph = gene.graph.clone();
        graph.retain_nodes(|_, node| steiner_tree.nodes[ctx.to_node_ix(node).index()]);
        for terminal in &gene.terminals {
            assert!(petgraph::algo::has_path_connecting(&graph, gene.root, *terminal, None));
        }

        // Sanity check we're not too far off.
        // GreedyFlac is a simple greedy algorithm, so we allow a wider range
        assert!((1.0..3.0).contains(&ratio), "{} {ratio}", gene.name);
        assert!(
            prepare_duration + grow_duration < Duration::from_millis(200),
            "{}",
            gene.name
        );

        all_results.push(DatasetResults {
            name: gene.name,
            algorithm: "GreedyFlac",
            main_result,
        });
    }

    let report = TestReport {
        algorithm: "GreedyFlac",
        results: all_results,
    };
    println!("{report}");

    for result in &report.results {
        assert!(
            (result.main_result.cost as f64 / result.main_result.optimal_cost as f64) <= 1.05,
            "Cost difference is too big for {}",
            result.name,
        );
    }
}
