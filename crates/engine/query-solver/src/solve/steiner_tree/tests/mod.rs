mod gene;

use std::fmt;
use std::time::{Duration, Instant};

use petgraph::visit::{EdgeRef, IntoNodeReferences};

use crate::Cost;
use crate::solve::steiner_tree::SteinerContext;

use super::{GreedyFlacAlgorithm, ShortestPathAlgorithm};

#[derive(Debug)]
struct AlgorithmResult {
    cost: Cost,
    optimal_cost: Cost,
    node_count: usize,
    kept_nodes_percentage: f64,
    prepare_duration: Duration,
    grow_duration: Duration,
}

#[derive(Debug)]
struct QuickEstimateResult {
    cost: Cost,
    optimal_cost: Cost,
    duration: Duration,
}

#[derive(Debug)]
struct DatasetResults {
    name: &'static str,
    algorithm: &'static str,
    main_result: AlgorithmResult,
    quick_estimate: QuickEstimateResult,
}

impl fmt::Display for DatasetResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{} - {}", self.algorithm, self.name)?;
        writeln!(f, "{}", "=".repeat(40))?;
        writeln!(f, "Main algorithm:")?;
        let diff = self.main_result.cost - self.main_result.optimal_cost;
        writeln!(
            f,
            "  Cost difference:    {} ({:+.1}%)",
            diff,
            (diff as f64 / self.main_result.optimal_cost as f64) * 100.0
        )?;
        writeln!(
            f,
            "  Nodes in tree:      {} ({:.0}% of graph)",
            self.main_result.node_count, self.main_result.kept_nodes_percentage
        )?;
        writeln!(f, "  Preparation time:   {:?}", self.main_result.prepare_duration)?;
        writeln!(f, "  Growth time:        {:?}", self.main_result.grow_duration)?;
        writeln!(
            f,
            "  Total time:         {:?}",
            self.main_result.prepare_duration + self.main_result.grow_duration
        )?;
        writeln!(f)?;
        writeln!(f, "Quick estimate:")?;
        let diff = self.quick_estimate.cost - self.quick_estimate.optimal_cost;
        writeln!(
            f,
            "  Cost difference:    {} ({:+.1}%)",
            diff,
            (diff as f64 / self.quick_estimate.optimal_cost as f64) * 100.0
        )?;
        writeln!(f, "  Time:               {:?}", self.quick_estimate.duration)?;
        Ok(())
    }
}

struct TestReport {
    algorithm: &'static str,
    results: Vec<DatasetResults>,
}

impl fmt::Display for TestReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{} ALGORITHM RESULTS", self.algorithm.to_uppercase())?;
        writeln!(f, "{}", "=".repeat(60))?;

        for result in &self.results {
            write!(f, "{result}")?;
        }

        writeln!(f, "\nSummary for {}:", self.algorithm)?;
        writeln!(f, "  Datasets tested: {}", self.results.len())?;

        let avg_diff: f64 = self
            .results
            .iter()
            .map(|r| (r.main_result.cost - r.main_result.optimal_cost) as f64)
            .sum::<f64>()
            / self.results.len() as f64;
        let avg_optimal: f64 = self
            .results
            .iter()
            .map(|r| r.main_result.optimal_cost as f64)
            .sum::<f64>()
            / self.results.len() as f64;
        writeln!(
            f,
            "  Average cost difference: {:.1} ({:+.1}%)",
            avg_diff,
            (avg_diff / avg_optimal) * 100.0
        )?;

        let total_time: Duration = self
            .results
            .iter()
            .map(|r| r.main_result.prepare_duration + r.main_result.grow_duration)
            .sum();
        writeln!(f, "  Total time: {total_time:?}")?;

        Ok(())
    }
}

/// Sanity check our ShortestPath Steiner Tree algorithm against a graph with known optimal cost.
#[test]
fn shortest_path_steinlib_gene() {
    let mut all_results = Vec::new();

    for gene in gene::load_dataset() {
        let start = Instant::now();
        let mut alg = ShortestPathAlgorithm::initialize(
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
        assert!((1.0..1.5).contains(&ratio), "{} {ratio}", gene.name);
        assert!(
            prepare_duration + grow_duration < Duration::from_millis(100),
            "{}",
            gene.name
        );

        // all terminals are in the steiner tree, so should be free.
        assert_eq!(alg.estimate_extra_cost(&[], &gene.terminals), 0);
        // We should have the same result with this method. The only difference is that the one
        // before supports changing the cost and adding terminals between iterations, which we didn't
        // do.
        let mut alg2 = ShortestPathAlgorithm::initialize(
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
        assert!(total_cost <= quick_cost, "{total_cost} <= quick_cost");
        let ratio = (quick_cost as f64) / (gene.optimal_cost as f64);
        assert!((1.0..1.5).contains(&ratio), "{} {ratio}", gene.name);

        let quick_estimate = QuickEstimateResult {
            cost: quick_cost,
            optimal_cost: gene.optimal_cost,
            duration: start.elapsed(),
        };

        all_results.push(DatasetResults {
            name: gene.name,
            algorithm: "ShortestPath",
            main_result,
            quick_estimate,
        });
    }

    let report = TestReport {
        algorithm: "ShortestPath",
        results: all_results,
    };
    println!("{report}");
}

/// Sanity check our GreedyFlac Steiner Tree algorithm against a graph with known optimal cost.
#[test]
fn greedy_flac_steinlib_gene() {
    let mut all_results = Vec::new();

    for gene in gene::load_dataset() {
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
}
