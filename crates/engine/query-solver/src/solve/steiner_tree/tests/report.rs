use std::{fmt, time::Duration};

use crate::solve::input::SteinerWeight;

#[derive(Debug)]
pub(super) struct AlgorithmRunReport {
    pub name: &'static str,
    pub algorithm: &'static str,
    pub weight: SteinerWeight,
    pub optimal_weight: SteinerWeight,
    pub node_count: usize,
    pub kept_nodes_percentage: f64,
    pub prepare_duration: Duration,
    pub grow_duration: Duration,
}

impl fmt::Display for AlgorithmRunReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{} - {}", self.algorithm, self.name)?;
        writeln!(f, "{}", "=".repeat(40))?;
        let diff = self.weight - self.optimal_weight;
        writeln!(
            f,
            "  Weight difference:    {} ({:+.1}%)",
            diff,
            (diff as f64 / self.optimal_weight as f64) * 100.0
        )?;
        writeln!(
            f,
            "  Nodes in tree:      {} ({:.0}% of graph)",
            self.node_count, self.kept_nodes_percentage
        )?;
        writeln!(f, "  Preparation time:   {:?}", self.prepare_duration)?;
        writeln!(f, "  Growth time:        {:?}", self.grow_duration)?;
        writeln!(
            f,
            "  Total time:         {:?}",
            self.prepare_duration + self.grow_duration
        )?;
        Ok(())
    }
}

pub(super) struct TestReport {
    pub algorithm: &'static str,
    pub reports: Vec<AlgorithmRunReport>,
}

impl fmt::Display for TestReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{} ALGORITHM RESULTS", self.algorithm.to_uppercase())?;
        writeln!(f, "{}", "=".repeat(60))?;

        for result in &self.reports {
            write!(f, "{result}")?;
        }

        writeln!(f, "\nSummary for {}:", self.algorithm)?;
        writeln!(f, "  Datasets tested: {}", self.reports.len())?;

        let avg_diff: f64 = self
            .reports
            .iter()
            .map(|r| (r.weight - r.optimal_weight) as f64)
            .sum::<f64>()
            / self.reports.len() as f64;
        let avg_optimal: f64 =
            self.reports.iter().map(|r| r.optimal_weight as f64).sum::<f64>() / self.reports.len() as f64;
        writeln!(
            f,
            "  Average weight difference: {:.1} ({:+.1}%)",
            avg_diff,
            (avg_diff / avg_optimal) * 100.0
        )?;

        let total_time: Duration = self.reports.iter().map(|r| r.prepare_duration + r.grow_duration).sum();
        writeln!(f, "  Total time: {total_time:?}")?;

        Ok(())
    }
}
