use ::operation::Operation;
use operation::OperationContext;
use petgraph::{prelude::StableGraph, visit::EdgeRef};
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
/// # Context and Purpose
///
/// The Steiner tree algorithm is used in two contexts:
/// 1. For the actual solution: Finding the edges of the Steiner tree that determines the query plan
/// 2. For requirements estimation: Estimating the cost of requirements to determine how much weight
///    to add to edges (federated entity resolvers typically) if they require intermediate plans
///
/// # The Challenge of Requirements
///
/// Requirements are complicated to express for the Steiner tree problem because they are conditional
/// on the edges taken. The standard Steiner tree problem doesn't change weights midway through solving.
///
/// # The Solution Process
///
/// To deal with conditional requirements, we:
/// 1. Initial cost estimation: Estimate how costly certain edges are if they need extra requirements
///    (e.g., an ID field that wasn't requested)
/// 2. Run FLAC once: Create an initial subset of the final Steiner tree
/// 3. Update edge weights: Adjust weights based on what's already in the tree. If we're going to
///    the products subgraph through a federation entity request anyway, requesting a `upc` field becomes
///    free. Initially, we had no idea, so we assumed that request was costly and increased the weight
///    of any resolver requiring it.
/// 4. Add new terminals: If resolvers we chose have extra requirements not in the original query,
///    add them as terminals (nodes to reach) so they'll be in the final Steiner tree
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
            updater.initialize(ctx, &mut input, &mut steiner_tree)?;
            State::Unsolved { flac, updater }
        };

        let solver = Self {
            ctx,
            input,
            steiner_tree,
            state,
        };

        tracing::debug!("Steiner graph populated:\n{}", solver.to_pretty_dot_graph(false));

        Ok(solver)
    }

    pub(crate) fn solve(mut self) -> crate::Result<Solution<'schema>> {
        self.execute()?;
        Ok(self.into_solution())
    }

    /// Solves the Steiner tree problem for the resolvers of our operation graph.
    ///
    /// # The Core Algorithm Loop
    ///
    /// This executes the main solving loop that alternates between:
    /// 1. Tree growth (FLAC): Expands the Steiner tree to connect more terminals
    /// 2. Weight updates (Fixed-point algorithm): Adjusts edge weights based on new requirements
    ///
    /// The loop continues until both operations report no changes (reaching a fixed point).
    pub fn execute(&mut self) -> crate::Result<()> {
        match std::mem::take(&mut self.state) {
            State::Solved => {
                tracing::debug!("Steiner graph is already solved.");
            }
            State::Unsolved { mut flac, mut updater } => {
                loop {
                    // Grow the Steiner tree by running one iteration of GreedyFLAC
                    let growth = flac.run_once(&self.input.graph, &mut self.steiner_tree);

                    // Update edge weights based on requirements, potentially adding new terminals
                    // This runs the fixed-point algorithm until weights stabilize
                    let update = updater.run_fixed_point_weight(self.ctx, &mut self.input, &mut self.steiner_tree)?;

                    // Stop when both the tree growth and weight updates have stabilized
                    if update.is_break() && growth.is_break() {
                        break;
                    }

                    tracing::trace!("Solver step:\n{}", self.to_pretty_dot_graph(false));
                }
                tracing::debug!("Solver finished:\n{}", self.to_pretty_dot_graph(true));
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

    pub fn to_pretty_dot_graph(&self, steiner_tree_only: bool) -> String {
        use petgraph::dot::{Config, Dot};
        let mut graph: StableGraph<_, _> = self.input.graph.clone().into();
        if steiner_tree_only {
            graph = graph.filter_map(
                |node_id, _| if self.steiner_tree[node_id] { Some(()) } else { None },
                |edge_id, edge| if self.steiner_tree[edge_id] { Some(*edge) } else { None },
            );
        }
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &graph,
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
    pub fn to_dot_graph(&self, steiner_tree_only: bool) -> String {
        use petgraph::dot::{Config, Dot};
        if steiner_tree_only {
            let mut graph: StableGraph<_, _> = self.input.graph.clone().into();
            graph = graph.filter_map(
                |node_id, _| if self.steiner_tree[node_id] { Some(()) } else { None },
                |edge_id, edge| if self.steiner_tree[edge_id] { Some(*edge) } else { None },
            );
            format!(
                "{:?}",
                Dot::with_attr_getters(
                    &graph,
                    &[Config::EdgeNoLabel, Config::NodeNoLabel],
                    &|_, edge| {
                        let weight = *edge.weight();
                        format!("cost={weight}")
                    },
                    &|_, (node_id, _)| {
                        let space_node_id = self.input.to_space_node_id(node_id);
                        Attrs::label(
                            self.input
                                .space
                                .graph
                                .node_weight(space_node_id)
                                .unwrap()
                                .label(&self.input.space, self.ctx),
                        )
                        .to_string()
                    }
                )
            )
        } else {
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
}

impl std::fmt::Debug for Solver<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Solver").finish_non_exhaustive()
    }
}
