use ::operation::Operation;
use operation::OperationContext;
use petgraph::visit::EdgeRef;
use schema::Schema;

use crate::{
    QuerySolutionSpace, SpaceNode,
    dot_graph::Attrs,
    solve::{
        Solution,
        input::{SteinerInput, build_input_and_terminals},
        steiner_tree::SteinerTree,
        updater::RequirementAndWeightUpdater,
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
    state: State,
}

#[allow(clippy::large_enum_variant)]
#[derive(Default)]
enum State {
    Unsolved {
        flac: GreedyFlac,
        updater: RequirementAndWeightUpdater,
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
        let (mut input, mut steiner_tree) = build_input_and_terminals(ctx, query_solution_space)?;

        let state = if steiner_tree.terminals.is_empty() {
            State::Solved
        } else {
            let flac = GreedyFlac::new(&input.graph);
            let mut updater = RequirementAndWeightUpdater::new(&input)?;
            let _ = updater.run_fixed_point_weight(&mut input, &mut steiner_tree)?;
            State::Unsolved { flac, updater }
        };

        let solver = Self {
            ctx,
            input,
            steiner_tree,
            state,
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
            State::Solved => {
                tracing::debug!("Steiner graph is already solved.");
            }
            State::Unsolved { mut flac, mut updater } => {
                loop {
                    let growth = flac.run_once(&self.input.graph, &mut self.steiner_tree);
                    let update = updater.run_fixed_point_weight(&mut self.input, &mut self.steiner_tree)?;

                    if update.is_break() && growth.is_break() {
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
                    let weight = *edge.weight();
                    let label = if weight > 0 { format!("{weight}") } else { String::new() };
                    // // Often useful for debugging, but makes the graph a lot harder to read.
                    // let space_edge_id = self.input.map.edge_id_to_space_edge_id[edge.id().index()];
                    // let (src, dst) = self.input.space.graph.edge_endpoints(space_edge_id).unwrap();
                    // let label = format!(
                    //     "{} {} -> {}",
                    //     label,
                    //     self.input.space.graph[src].label(&self.input.space, self.ctx),
                    //     self.input.space.graph[dst].label(&self.input.space, self.ctx)
                    // );
                    Attrs::label(label)
                        .with_if(is_in_steiner_tree, "color=forestgreen,fontcolor=forestgreen")
                        .with_if(!is_in_steiner_tree, "color=royalblue,fontcolor=royalblue,style=dashed")
                        .to_string()
                },
                &|_, (node_id, _)| {
                    let is_in_steiner_tree = self.steiner_tree[node_id];
                    let is_leaf = self
                        .input
                        .graph
                        .edges_directed(node_id, petgraph::Direction::Outgoing)
                        .count()
                        == 0;
                    let is_terminal = self.steiner_tree.terminals.contains(&node_id);
                    let space_node_id = self.input.to_space_node_id(node_id);
                    let weight = self.input.space.graph.node_weight(space_node_id).unwrap();
                    let attrs = Attrs::label(weight.label(&self.input.space, self.ctx));
                    let attrs = match weight {
                        SpaceNode::ProvidableField(_) => attrs.with("shape=box"),
                        SpaceNode::Resolver(_) => attrs.with("shape=parallelogram"),
                        _ => attrs,
                    };
                    match (is_in_steiner_tree, is_leaf, is_terminal) {
                        (true, _, true) => attrs.with("color=forestgreen style=bold"),
                        (true, _, false) => attrs.with("color=forestgreen"),
                        (false, _, true) => attrs,
                        (false, true, false) => attrs.with("style=dashed"),
                        (false, false, false) => attrs.with("color=royalblue style=dashed"),
                    }
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
                    let weight = *edge.weight();
                    format!("cost={weight}, steiner={}", is_in_steiner_tree as usize)
                },
                &|_, (node_id, _)| {
                    let is_in_steiner_tree = self.steiner_tree[node_id];
                    let space_node_id = self.input.to_space_node_id(node_id);
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
