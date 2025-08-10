use std::{fmt, time::Duration};

use crate::Cost;

#[derive(Debug)]
pub(super) struct AlgorithmResult {
    pub cost: Cost,
    pub optimal_cost: Cost,
    pub node_count: usize,
    pub kept_nodes_percentage: f64,
    pub prepare_duration: Duration,
    pub grow_duration: Duration,
}

#[derive(Debug)]
pub(super) struct DatasetResults {
    pub name: &'static str,
    pub algorithm: &'static str,
    pub main_result: AlgorithmResult,
}

impl fmt::Display for DatasetResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{} - {}", self.algorithm, self.name)?;
        writeln!(f, "{}", "=".repeat(40))?;
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
        Ok(())
    }
}

pub(super) struct TestReport {
    pub algorithm: &'static str,
    pub results: Vec<DatasetResults>,
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
