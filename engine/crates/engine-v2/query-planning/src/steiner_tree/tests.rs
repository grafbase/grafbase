mod gene;

use std::time::{Duration, Instant};

use super::{shortest_path::ShortestPathAlg, SteinerTreeAlg};

/// Sanity check our ShortestPath Steiner Tree algorithm against a graph with known optimal cost.
#[test]
fn shortest_path_steinlib_gene() {
    for gene in gene::load_dataset() {
        let start = Instant::now();
        let mut alg = ShortestPathAlg::init(&gene.graph, gene.root, gene.terminals.clone());
        let mut solution = loop {
            if let Some(solution) = alg.grow_steiner_tree(|edge| *edge) {
                break solution;
            }
        };

        let ratio = (solution.total_cost as f64) / (gene.optimal_cost as f64);
        let duration = start.elapsed();
        println!(
            "{} | ratio: {ratio:.5} | kept nodes: {:.0}% | {duration:?}",
            gene.name,
            ((solution.steiner_tree_nodes.count_ones() * 100) as f64) / (gene.graph.node_count() as f64)
        );

        // We found all the terminals.
        solution.terminals.sort_unstable();
        assert_eq!(gene.terminals, solution.terminals);
        assert!(gene
            .terminals
            .iter()
            .all(|terminal| solution.steiner_tree_nodes[terminal.index()]));

        // Are all the terminals accessible from root?
        let mut graph = gene.graph;
        graph.retain_nodes(|_, node| solution.steiner_tree_nodes[node.index()]);
        for terminal in gene.terminals {
            assert!(petgraph::algo::has_path_connecting(&graph, gene.root, terminal, None));
        }

        // Sanity check we're not too far off.
        assert!(ratio < 1.5, "{} {ratio}", gene.name);
        assert!(duration < Duration::from_millis(100), "{}", gene.name);
    }
}
