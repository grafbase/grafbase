use ::operation::Operation;
use fixedbitset::FixedBitSet;
use itertools::Itertools as _;
use operation::OperationContext;
use petgraph::visit::EdgeRef;
use schema::Schema;

use crate::{
    QuerySolutionSpace, SolutionSpaceGraph,
    dot_graph::Attrs,
    solve::{
        context::{SteinerContext, SteinerGraph},
        requirements::RequirementAndCostUpdater,
        steiner_tree::SteinerTree,
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
pub(crate) struct Solver<'schema, 'op, 'q> {
    schema: &'schema Schema,
    operation: &'op Operation,
    query_solution_space: &'q QuerySolutionSpace<'schema>,
    ctx: SteinerContext<&'q SolutionSpaceGraph<'schema>, SteinerGraph>,
    flac: GreedyFlac,
    steiner_tree: SteinerTree,
    requirements_and_cost_updater: RequirementAndCostUpdater,
}

pub(crate) struct SteinerTreeSolution {
    pub node_bitset: FixedBitSet,
}

impl<'schema, 'op, 'q> Solver<'schema, 'op, 'q>
where
    'schema: 'op,
{
    pub(crate) fn initialize(
        schema: &'schema Schema,
        operation: &'op Operation,
        query_solution_space: &'q QuerySolutionSpace<'schema>,
    ) -> crate::Result<Self> {
        let (ctx, terminals) = SteinerContext::from_query_solution_space(query_solution_space);

        let steiner_tree = SteinerTree::new(&ctx.graph, ctx.root_ix);
        let flac = GreedyFlac::new(&ctx.graph, terminals);
        let cost_updater = RequirementAndCostUpdater::new(&ctx)?;

        let mut solver = Self {
            schema,
            operation,
            query_solution_space,
            ctx,
            flac,
            steiner_tree,
            requirements_and_cost_updater: cost_updater,
        };

        let new_terminals = solver
            .requirements_and_cost_updater
            .run_fixed_point_cost(&mut solver.ctx.graph, &solver.steiner_tree)?;
        debug_assert!(
            new_terminals.is_empty(),
            "Fixed point cost algorithm should not return new terminals at initialization"
        );

        tracing::debug!("Solver populated:\n{}", solver.to_pretty_dot_graph());

        Ok(solver)
    }

    pub(crate) fn solve(mut self) -> crate::Result<SteinerTreeSolution> {
        self.execute()?;
        Ok(self.into_solution())
    }

    /// Solves the Steiner tree problem for the resolvers of our operation graph.
    pub fn execute(&mut self) -> crate::Result<()> {
        loop {
            let growth = self.flac.run_once(&self.ctx.graph, &mut self.steiner_tree);
            let new_terminals = self
                .requirements_and_cost_updater
                .run_fixed_point_cost(&mut self.ctx.graph, &self.steiner_tree)?;

            println!("NEW TERMINALS: {}", new_terminals.len());
            if growth.is_break() && new_terminals.is_empty() {
                break;
            }

            new_terminals.sort_unstable();
            self.flac
                .extend_terminals(new_terminals.drain(..).dedup().filter(|idx| !self.steiner_tree[*idx]));

            tracing::trace!("Solver step:\n{}", self.to_pretty_dot_graph());
        }
        tracing::debug!("Solver finished:\n{}", self.to_pretty_dot_graph());

        Ok(())
    }

    pub fn into_solution(self) -> SteinerTreeSolution {
        let mut bitset = FixedBitSet::with_capacity(self.ctx.space_graph_node_id_to_node_ix.len());
        for (i, ix) in self.ctx.space_graph_node_id_to_node_ix.iter().copied().enumerate() {
            bitset.set(i, self.steiner_tree[ix]);
        }
        SteinerTreeSolution { node_bitset: bitset }
    }

    pub fn to_pretty_dot_graph(&self) -> String {
        use petgraph::dot::{Config, Dot};
        let ctx = OperationContext {
            schema: self.schema,
            operation: self.operation,
        };
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.ctx.graph,
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
                    if let Some(node_id) = self.ctx.to_space_graph_node_id(node_ix) {
                        self.query_solution_space
                            .graph
                            .node_weight(node_id)
                            .unwrap()
                            .pretty_label(self.query_solution_space, ctx)
                            .with_if(!is_in_steiner_tree, "style=dashed")
                            .with_if(is_in_steiner_tree, "color=forestgreen")
                            .to_string()
                    } else {
                        "label=\"\", style=dashed".to_string()
                    }
                }
            )
        )
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    #[cfg(test)]
    pub fn to_dot_graph(&self) -> String {
        use petgraph::dot::{Config, Dot};
        let ctx = OperationContext {
            schema: self.schema,
            operation: self.operation,
        };
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.ctx.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_, edge| {
                    let is_in_steiner_tree = self.steiner_tree[edge.id()];
                    let cost = *edge.weight();
                    format!("cost={cost}, steiner={}", is_in_steiner_tree as usize)
                },
                &|_, (node_ix, _)| {
                    let is_in_steiner_tree = self.steiner_tree[node_ix];
                    if let Some(node_id) = self.ctx.to_space_graph_node_id(node_ix) {
                        Attrs::label(
                            self.query_solution_space
                                .graph
                                .node_weight(node_id)
                                .unwrap()
                                .label(self.query_solution_space, ctx),
                        )
                        .with(format!("steiner={}", is_in_steiner_tree as usize))
                        .to_string()
                    } else {
                        "label=\"\", style=dashed".to_string()
                    }
                }
            )
        )
    }
}

impl std::fmt::Debug for Solver<'_, '_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Solver").finish_non_exhaustive()
    }
}
