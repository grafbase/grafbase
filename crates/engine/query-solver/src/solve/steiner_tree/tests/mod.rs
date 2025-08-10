mod loader;
mod report;

use std::time::{Duration, Instant};

use petgraph::visit::{EdgeRef, IntoNodeReferences};

use crate::solve::steiner_tree::SteinerContext;

use super::GreedyFlacAlgorithm;
use report::*;

/// Sanity check our GreedyFlac Steiner Tree algorithm against a graph with known optimal cost.
#[test]
fn greedy_flac_steinlib_gene() {
    let mut all_results = Vec::new();

    for gene in loader::load_gene_dataset() {
        let start = Instant::now();
        let mut alg = GreedyFlacAlgorithm::initialize(
            SteinerContext::build(
                &gene.graph,
                gene.root,
                |(node, _)| Some(node),
                |edge| Some((edge.id(), edge.source(), edge.target(), *edge.weight())),
            ),
            gene.terminals.iter().copied(),
        );
        let prepare_duration = start.elapsed();

        let start = Instant::now();
        while alg.continue_steiner_tree_growth().is_continue() {}

        let total_cost = alg.total_cost();
        let steiner_tree_node_count = gene
            .graph
            .node_references()
            .filter(|(node_id, _)| alg.contains_node(*node_id))
            .count();

        let ratio = (total_cost as f64) / (gene.optimal_cost as f64);
        let grow_duration = start.elapsed();

        let main_result = AlgorithmResult {
            cost: total_cost,
            optimal_cost: gene.optimal_cost,
            node_count: steiner_tree_node_count,
            kept_nodes_percentage: ((steiner_tree_node_count * 100) as f64) / (gene.graph.node_count() as f64),
            prepare_duration,
            grow_duration,
        };

        assert!(gene.terminals.iter().all(|terminal| alg.contains_node(*terminal)));

        // Are all the terminals accessible from root?
        let mut graph = gene.graph.clone();
        graph.retain_nodes(|_, node| alg.contains_node(node));
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

        // all terminals are in the steiner tree, so should be free.
        assert_eq!(alg.estimate_extra_cost(&[], &gene.terminals), 0);

        // Test with a second algorithm instance
        let mut alg2 = GreedyFlacAlgorithm::initialize(
            SteinerContext::build(
                &gene.graph,
                gene.root,
                |(node, _)| Some(node),
                |edge| Some((edge.id(), edge.source(), edge.target(), *edge.weight())),
            ),
            gene.terminals.iter().copied(),
        );

        let start = Instant::now();
        let quick_cost = alg2.estimate_extra_cost(&[], &gene.terminals);
        assert!(total_cost <= quick_cost, "{total_cost} <= {quick_cost}");
        let ratio = (quick_cost as f64) / (gene.optimal_cost as f64);
        assert!((1.0..3.0).contains(&ratio), "{} {ratio}", gene.name);

        let quick_estimate = QuickEstimateResult {
            cost: quick_cost,
            optimal_cost: gene.optimal_cost,
            duration: start.elapsed(),
        };

        all_results.push(DatasetResults {
            name: gene.name,
            algorithm: "GreedyFlac",
            main_result,
            quick_estimate,
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
        assert!(
            result.main_result.cost == result.quick_estimate.cost,
            "Cost mismatch for {}",
            result.name,
        );
    }
}
