mod gene;

use std::time::{Duration, Instant};

use petgraph::visit::{EdgeRef, IntoNodeReferences};

use crate::steiner_tree::ShortestPathAlgorithm;

/// Sanity check our ShortestPath Steiner Tree algorithm against a graph with known optimal cost.
#[test]
fn shortest_path_steinlib_gene() {
    for gene in gene::load_dataset() {
        let start = Instant::now();
        let mut alg = ShortestPathAlgorithm::initialize(
            &gene.graph,
            |(node, _)| Some(node),
            |edge| Some((edge.id(), edge.source(), edge.target(), *edge.weight())),
            gene.root,
            gene.terminals.iter().copied(),
        );
        let prepare_duration = start.elapsed();

        let start = Instant::now();
        while alg.continue_steiner_tree_growth() {}

        let total_cost = alg.total_cost();
        let steiner_tree_node_count = gene
            .graph
            .node_references()
            .filter(|(node_id, _)| alg.contains_node(*node_id))
            .count();

        let ratio = (total_cost as f64) / (gene.optimal_cost as f64);
        let grow_duration = start.elapsed();
        println!(
            "{} | ratio: {ratio:.5} | kept nodes: {:.0}% | {prepare_duration:?}/{grow_duration:?}",
            gene.name,
            ((steiner_tree_node_count * 100) as f64) / (gene.graph.node_count() as f64),
        );

        assert!(gene.terminals.iter().all(|terminal| alg.contains_node(*terminal)));

        // Are all the terminals accessible from root?
        let mut graph = gene.graph.clone();
        graph.retain_nodes(|_, node| alg.contains_node(node));
        for terminal in &gene.terminals {
            assert!(petgraph::algo::has_path_connecting(&graph, gene.root, *terminal, None));
        }

        // Sanity check we're not too far off.
        assert!((1.0..1.5).contains(&ratio), "{} {ratio}", gene.name);
        assert!(
            prepare_duration + grow_duration < Duration::from_millis(100),
            "{}",
            gene.name
        );

        // all terminals are in the steiner tree, so should be free.
        assert_eq!(alg.estimate_extra_cost([], gene.terminals.iter().copied()), 0);
        // We should have the same result with this method. The only difference is that the one
        // before supports changing the cost and adding terminals between iterations, which we didn't
        // do.
        let mut alg2 = ShortestPathAlgorithm::initialize(
            &gene.graph,
            |(node, _)| Some(node),
            |edge| Some((edge.id(), edge.source(), edge.target(), *edge.weight())),
            gene.root,
            gene.terminals.iter().copied(),
        );

        let start = Instant::now();
        let quick_cost = alg2.estimate_extra_cost([], gene.terminals.iter().copied());
        assert!(total_cost <= quick_cost, "{total_cost} <= quick_cost");
        let ratio = (quick_cost as f64) / (gene.optimal_cost as f64);
        assert!((1.0..1.5).contains(&ratio), "{} {ratio}", gene.name);
        println!("{} | ratio: {ratio:.5} | {:?}", gene.name, start.elapsed());
    }
}
