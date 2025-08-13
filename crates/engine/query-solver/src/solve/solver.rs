use ::operation::Operation;
use itertools::Itertools as _;
use operation::OperationContext;
use petgraph::visit::EdgeRef;
use schema::Schema;

use crate::{
    QuerySolutionSpace,
    dot_graph::Attrs,
    solve::{
        Solution,
        input::{SteinerInput, build_input_and_terminals},
        steiner_tree::SteinerTree,
        updater::RequirementAndCostUpdater,
    },
};

use super::steiner_tree::GreedyFlac;

/// The solver is responsible for finding the optimal path from the root to the query fields.
/// There are two cores aspects to this, expressing the problem as a Steiner tree problem and
/// solving it with an appropriate algorithm.
///
/// For the first part, the most difficult aspect are dispensable requirements, meaning only needed
/// in certain paths.
/// We don't know whether we'll need them and we don't want to retrieve them if not necessary. To take them
/// into account, we adjust the cost of edges that require them. If requirements can be trivially
/// provided by the parent resolver, no cost is added. If it needs intermediate resolvers not (yet)
/// part of the Steiner it incurs an extra cost.
///
/// As this extra cost changes every time we change the Steiner tree, we have to adjust those while
/// constructing it.
pub(crate) struct Solver<'schema, 'op> {
    ctx: OperationContext<'op>,
    input: SteinerInput<'schema>,
    steiner_tree: SteinerTree,
    state: SolverState,
}

#[derive(Default)]
enum SolverState {
    SteinerTreeAlgorithm {
        flac: GreedyFlac,
        requirements_and_cost_updater: RequirementAndCostUpdater,
    },
    #[default]
    Solved,
}

impl<'schema, 'op> Solver<'schema, 'op>
where
    'schema: 'op,
{
    pub(crate) fn initialize(
        schema: &'schema Schema,
        operation: &'op Operation,
        query_solution_space: QuerySolutionSpace<'schema>,
    ) -> crate::Result<Self> {
        let ctx = OperationContext { schema, operation };
        let (mut input, terminals) = build_input_and_terminals(ctx, query_solution_space);

        let steiner_tree = SteinerTree::new(&input.graph, input.root_node_id);

        let algorithm = if terminals.is_empty() {
            SolverState::Solved
        } else {
            let flac = GreedyFlac::new(&input.graph, terminals);
            let mut requirements_and_cost_updater = RequirementAndCostUpdater::new(&input)?;
            let update = requirements_and_cost_updater.run_fixed_point_cost(&mut input, &steiner_tree)?;
            debug_assert!(
                update.new_terminals.is_empty(),
                "Fixed point cost algorithm should not return new terminals at initialization"
            );
            SolverState::SteinerTreeAlgorithm {
                flac,
                requirements_and_cost_updater,
            }
        };

        let solver = Self {
            ctx,
            input,
            steiner_tree,
            state: algorithm,
        };

        tracing::debug!("Steiner graph populated:\n{}", solver.to_pretty_dot_graph());

        Ok(solver)
    }

    pub(crate) fn solve(mut self) -> crate::Result<Solution<'schema>> {
        self.execute()?;
        Ok(self.into_solution())
    }

    /// Solves the Steiner tree problem for the resolvers of our operation graph.
    pub fn execute(&mut self) -> crate::Result<()> {
        match std::mem::take(&mut self.state) {
            SolverState::Solved => {
                tracing::debug!("Steiner graph is already solved.");
            }
            SolverState::SteinerTreeAlgorithm {
                mut flac,
                mut requirements_and_cost_updater,
            } => {
                loop {
                    let growth = flac.run_once(&self.input.graph, &mut self.steiner_tree);
                    let update =
                        requirements_and_cost_updater.run_fixed_point_cost(&mut self.input, &self.steiner_tree)?;

                    if !update.new_terminals.is_empty() {
                        update.new_terminals.sort_unstable();
                        flac.extend_terminals(update.new_terminals.drain(..).dedup());
                    } else if growth.is_break() {
                        break;
                    }

                    tracing::trace!("Solver step:\n{}", self.to_pretty_dot_graph());
                }
                tracing::debug!("Solver finished:\n{}", self.to_pretty_dot_graph());
            }
        }

        Ok(())
    }

    pub fn into_solution(self) -> Solution<'schema> {
        Solution {
            input: self.input,
            steiner_tree: self.steiner_tree,
        }
    }

    pub fn to_pretty_dot_graph(&self) -> String {
        use petgraph::dot::{Config, Dot};
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.input.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| {
                    let is_in_steiner_tree = self.steiner_tree[edge.id()];
                    let cost = *edge.weight();
                    Attrs::label_if(cost > 0, cost.to_string())
                        .bold()
                        .with_if(is_in_steiner_tree, "color=forestgreen,fontcolor=forestgreen")
                        .with_if(!is_in_steiner_tree, "color=royalblue,fontcolor=royalblue,style=dashed")
                        .to_string()
                },
                &|_, (node_ix, _)| {
                    let is_in_steiner_tree = self.steiner_tree[node_ix];
                    let space_node_id = self.input.to_space_node_id(node_ix);
                    self.input
                        .space
                        .graph
                        .node_weight(space_node_id)
                        .unwrap()
                        .pretty_label(&self.input.space, self.ctx)
                        .with_if(!is_in_steiner_tree, "style=dashed")
                        .with_if(is_in_steiner_tree, "color=forestgreen")
                        .to_string()
                }
            )
        )
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    #[cfg(test)]
    pub fn to_dot_graph(&self) -> String {
        use petgraph::dot::{Config, Dot};
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.input.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| {
                    let is_in_steiner_tree = self.steiner_tree[edge.id()];
                    let cost = *edge.weight();
                    format!("cost={cost}, steiner={}", is_in_steiner_tree as usize)
                },
                &|_, (node_ix, _)| {
                    let is_in_steiner_tree = self.steiner_tree[node_ix];
                    let space_node_id = self.input.to_space_node_id(node_ix);
                    Attrs::label(
                        self.input
                            .space
                            .graph
                            .node_weight(space_node_id)
                            .unwrap()
                            .label(&self.input.space, self.ctx),
                    )
                    .with(format!("steiner={}", is_in_steiner_tree as usize))
                    .to_string()
                }
            )
        )
    }
}

impl std::fmt::Debug for Solver<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Solver").finish_non_exhaustive()
    }
}
