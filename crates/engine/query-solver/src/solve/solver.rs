use std::{
    hash::{BuildHasher, Hash},
    ops::ControlFlow,
};

use ::operation::Operation;
use fixedbitset::FixedBitSet;
use fxhash::FxBuildHasher;
use hashbrown::hash_table::Entry;
use id_newtypes::IdRange;
use itertools::Itertools;
use operation::OperationContext;
use petgraph::{
    Direction,
    prelude::StableGraph,
    stable_graph::{EdgeIndex, EdgeReference, NodeIndex},
    visit::{EdgeRef, IntoNodeReferences},
};
use schema::Schema;

use crate::{
    Cost, FieldFlags, QuerySolutionSpace,
    dot_graph::Attrs,
    solution_space::{SpaceEdge, SpaceNode},
    solve::steiner_tree::SteinerContext,
};

use super::steiner_tree::{self, SteinerGraph};

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
    algorithm: steiner_tree::GreedyFlacAlgorithm<&'q StableGraph<SpaceNode<'schema>, SpaceEdge>, SteinerGraph>,
    /// Keeps track of dispensable requirements to adjust edge cost, ideally we'd like to avoid
    /// them.
    dispensable_requirements_metadata: DispensableRequirementsMetadata,
    /// Temporary storage for extra terminals to be added to the algorithm.
    tmp_extra_terminals: Vec<NodeIndex>,
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
        let mut terminals = Vec::new();
        for (node_ix, node) in query_solution_space.graph.node_references() {
            if let SpaceNode::QueryField(field) = node
                && field.flags.contains(FieldFlags::LEAF_NODE | FieldFlags::INDISPENSABLE)
            {
                terminals.push(node_ix);
            }
        }
        let node_filter = |(node_ix, node): (NodeIndex, &SpaceNode<'schema>)| match node {
            SpaceNode::Root | SpaceNode::Resolver(_) | SpaceNode::ProvidableField(_) => Some(node_ix),
            SpaceNode::QueryField(field) => {
                if field.is_leaf() {
                    Some(node_ix)
                } else {
                    None
                }
            }
        };
        let edge_filter = |edge: EdgeReference<'_, SpaceEdge, _>| match edge.weight() {
            // Resolvers have an inherent cost of 1.
            SpaceEdge::CreateChildResolver => Some((edge.id(), edge.source(), edge.target(), 1)),
            SpaceEdge::CanProvide | SpaceEdge::Provides | SpaceEdge::TypenameField => {
                Some((edge.id(), edge.source(), edge.target(), 0))
            }
            SpaceEdge::Field | SpaceEdge::HasChildResolver | SpaceEdge::Requires => None,
        };

        let algorithm = steiner_tree::GreedyFlacAlgorithm::initialize(
            SteinerContext::build(
                &query_solution_space.graph,
                query_solution_space.root_node_ix,
                node_filter,
                edge_filter,
            ),
            terminals,
        );

        let mut solver = Self {
            schema,
            operation,
            query_solution_space,
            algorithm,
            dispensable_requirements_metadata: DispensableRequirementsMetadata::default(),
            tmp_extra_terminals: Vec::new(),
        };

        solver.populate_requirement_metadata()?;
        let _ = solver.cost_fixed_point_iteration()?;

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
            let growth = self.algorithm.continue_steiner_tree_growth();
            let cost_update = self.cost_fixed_point_iteration()?;
            if growth.is_break() && cost_update.is_break() {
                break;
            }
            tracing::trace!("Solver step:\n{}", self.to_pretty_dot_graph());
        }
        tracing::debug!("Solver finished:\n{}", self.to_pretty_dot_graph());

        Ok(())
    }

    pub fn into_solution(self) -> SteinerTreeSolution {
        SteinerTreeSolution {
            node_bitset: self.algorithm.into_query_graph_nodes_bitset(),
        }
    }

    /// For each node with dispensable requirements, we need its incoming edge's cost to reflect
    /// the requirements cost if we were to chose that edge. Those dispensable requirements would then become
    /// indispensable and added to the list of terminals we must find in the Steiner tree.
    ///
    /// A node may have multiple incoming edges being potentially resolved by different resolvers.
    /// This may have implications on the requirements, so we recursively consider any parent incoming edge to
    /// be free as long as there is only one parent. We had to take that path after all. This
    /// allow us to more appropriately reflect cost differences.
    ///
    /// This method populates all the necessary metadata used to compute the extra requirements cost.
    fn populate_requirement_metadata(&mut self) -> crate::Result<()> {
        struct IncomingEdgeWithDispensableRequirements {
            parent: NodeIndex,
            extra_required_node_ids: IdRange<ExtraRequiredNodeId>,
            incoming_edge_ix: EdgeIndex,
            edge_cost: Cost,
        }
        let mut buffer = Vec::with_capacity(self.query_solution_space.graph.node_count() >> 4);

        // Used to intern required node id ranges
        let hasher = FxBuildHasher::default();
        let mut requirements_interner = hashbrown::HashTable::<IdRange<ExtraRequiredNodeId>>::with_capacity(
            self.query_solution_space.graph.node_count() >> 4,
        );

        for (node_ix, node) in self.query_solution_space.graph.node_references() {
            if !matches!(node, SpaceNode::Resolver(_) | SpaceNode::ProvidableField(_)) {
                continue;
            }

            // Retrieve all the node ids on which we depend.
            let extra_required_node_ids = self.dispensable_requirements_metadata.extend_extra_required_nodes(
                self.query_solution_space
                    .graph
                    .edges_directed(node_ix, Direction::Outgoing)
                    .filter(|edge| {
                        matches!(edge.weight(), SpaceEdge::Requires)
                            && self.query_solution_space.graph[edge.target()]
                                .as_query_field()
                                .map(|field| !field.is_indispensable() && field.is_leaf())
                                .unwrap_or_default()
                    })
                    .map(|edge| edge.target()),
            );
            if extra_required_node_ids.is_empty() {
                continue;
            }

            // De-duplicate the requirements
            let key = &self.dispensable_requirements_metadata[extra_required_node_ids];
            let extra_required_node_ids = match requirements_interner.entry(
                hasher.hash_one(key),
                |id| &self.dispensable_requirements_metadata[*id] == key,
                |id| hasher.hash_one(&self.dispensable_requirements_metadata[*id]),
            ) {
                Entry::Occupied(entry) => {
                    self.dispensable_requirements_metadata
                        .extra_required_nodes
                        .truncate(extra_required_node_ids.start.into());

                    *entry.get()
                }
                Entry::Vacant(entry) => {
                    entry.insert(extra_required_node_ids);
                    extra_required_node_ids
                }
            };

            // Given a parent node, if there is a ProvidableField neighbor that provides our field
            // without any requirements, there is no cost associated with it.
            // If for each parent all the requirements have no cost, there is no extra cost at all
            // for this field.
            if self
                .query_solution_space
                .graph
                .edges_directed(node_ix, Direction::Incoming)
                .filter(|edge| matches!(edge.weight(), SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide))
                .all(|incoming_edge| {
                    let parent = incoming_edge.source();
                    self.dispensable_requirements_metadata[extra_required_node_ids]
                        .iter()
                        .all(|required| {
                            self.query_solution_space
                                .graph
                                .edges_directed(parent, Direction::Outgoing)
                                .filter(|neighbor| matches!(neighbor.weight(), SpaceEdge::CanProvide))
                                .any(|neighbor| {
                                    let mut found_requirement = false;
                                    for edge in self
                                        .query_solution_space
                                        .graph
                                        .edges_directed(neighbor.target(), Direction::Outgoing)
                                    {
                                        if matches!(edge.weight(), SpaceEdge::Requires) {
                                            return false;
                                        }
                                        found_requirement |=
                                            matches!(edge.weight(), SpaceEdge::Provides) & (edge.target() == *required);
                                    }
                                    found_requirement
                                })
                        })
                })
            {
                self.dispensable_requirements_metadata
                    .free_requirements
                    .push((node_ix, extra_required_node_ids));
                continue;
            }

            for incoming_edge in self
                .query_solution_space
                .graph
                .edges_directed(node_ix, Direction::Incoming)
            {
                let edge_cost = match incoming_edge.weight() {
                    SpaceEdge::CreateChildResolver => 1,
                    SpaceEdge::CanProvide => 0,
                    _ => continue,
                };
                buffer.push(IncomingEdgeWithDispensableRequirements {
                    parent: incoming_edge.source(),
                    extra_required_node_ids,
                    incoming_edge_ix: incoming_edge.id(),
                    edge_cost,
                });
            }
        }

        buffer.sort_unstable_by(|a, b| {
            a.parent
                .cmp(&b.parent)
                .then(a.extra_required_node_ids.cmp(&b.extra_required_node_ids))
        });

        for ((mut parent, extra_required_node_ids), chunk) in buffer
            .into_iter()
            .chunk_by(|item| (item.parent, item.extra_required_node_ids))
            .into_iter()
        {
            let incoming_edge_and_cost_ids = self
                .dispensable_requirements_metadata
                .extend_incoming_edges_and_cost(chunk.into_iter().map(|item| (item.incoming_edge_ix, item.edge_cost)));

            // This will at least include the ProvidableField & Resolver that led to the
            // parent. As we'll necessarily take them for this particular edge, they'll be set
            // to 0 cost while estimating the requirement cost.
            let zero_cost_parent_edge_ids =
                self.dispensable_requirements_metadata
                    .extend_zero_cost_parent_edges(std::iter::from_fn(|| {
                        let mut grand_parents = self
                            .query_solution_space
                            .graph
                            .edges_directed(parent, Direction::Incoming)
                            .filter(|edge| {
                                matches!(edge.weight(), SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide)
                            });

                        let first = grand_parents.next()?;
                        if grand_parents.next().is_none() {
                            parent = first.source();
                            Some(first.id())
                        } else {
                            None
                        }
                    }));

            self.dispensable_requirements_metadata
                .maybe_costly_requirements
                .push(DispensableRequirements {
                    zero_cost_parent_edge_ids,
                    extra_required_node_ids,
                    incoming_edge_and_cost_ids,
                });
        }

        Ok(())
    }

    /// Updates the cost of edges based on the requirements of the nodes.
    /// We iterate until cost becomes stable or we exhausted the maximum number of iterations which
    /// likely indicates a requirement cycle.
    fn cost_fixed_point_iteration(&mut self) -> crate::Result<ControlFlow<()>> {
        debug_assert!(self.tmp_extra_terminals.is_empty());
        let mut i = 0;
        loop {
            i += 1;
            self.generate_cost_updates_based_on_requirements();
            if !self.algorithm.apply_all_cost_updates()
                || self
                    .dispensable_requirements_metadata
                    .independent_cost
                    .unwrap_or_default()
            {
                break;
            }
            if i > 100 {
                return Err(crate::Error::RequirementCycleDetected);
            }
        }
        // If it's the first time we do the fixed point iteration and we didn't do more than 2
        // iterations (one for updating, one for checking nothing changed). It means there is no
        // dependency between requirements cost. So we can skip it in the next iterations.
        self.dispensable_requirements_metadata
            .independent_cost
            .get_or_insert(i == 2);
        let new_terminals = !self.tmp_extra_terminals.is_empty();
        self.tmp_extra_terminals.sort_unstable();
        self.algorithm
            .extend_terminals(self.tmp_extra_terminals.drain(..).dedup());

        Ok(if new_terminals {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        })
    }

    /// For all edges with dispensable requirements, we estimate the cost of the extra requirements
    /// by computing cost of adding them to the current Steiner tree plus the base cost of the
    /// edge.
    fn generate_cost_updates_based_on_requirements(&mut self) {
        let mut i = 0;
        while let Some((node_id, extra_required_node_ids)) =
            self.dispensable_requirements_metadata.free_requirements.get(i).copied()
        {
            if self.algorithm.contains_node(node_id) {
                self.tmp_extra_terminals.extend(
                    self.dispensable_requirements_metadata[extra_required_node_ids]
                        .iter()
                        .copied(),
                );
                self.dispensable_requirements_metadata.free_requirements.swap_remove(i);
            } else {
                i += 1;
            }
        }

        i = 0;
        while let Some(DispensableRequirements {
            extra_required_node_ids,
            zero_cost_parent_edge_ids,
            incoming_edge_and_cost_ids,
        }) = self
            .dispensable_requirements_metadata
            .maybe_costly_requirements
            .get(i)
            .copied()
        {
            if self.dispensable_requirements_metadata[incoming_edge_and_cost_ids]
                .iter()
                .any(|(incoming_edge, _)| {
                    let (_, target_ix) = self.query_solution_space.graph.edge_endpoints(*incoming_edge).unwrap();
                    self.algorithm.contains_node(target_ix)
                })
            {
                self.tmp_extra_terminals.extend(
                    self.dispensable_requirements_metadata[extra_required_node_ids]
                        .iter()
                        .copied(),
                );
                self.dispensable_requirements_metadata
                    .maybe_costly_requirements
                    .swap_remove(i);
                continue;
            }

            let extra_cost = self.algorithm.estimate_extra_cost(
                &self.dispensable_requirements_metadata[zero_cost_parent_edge_ids],
                &self.dispensable_requirements_metadata[extra_required_node_ids],
            );

            for (incoming_edge, cost) in &self.dispensable_requirements_metadata[incoming_edge_and_cost_ids] {
                let (source_ix, _) = self.query_solution_space.graph.edge_endpoints(*incoming_edge).unwrap();
                self.algorithm
                    .insert_edge_cost_update(source_ix, *incoming_edge, cost + extra_cost);
            }

            i += 1;
        }
    }

    pub fn to_pretty_dot_graph(&self) -> String {
        let ctx = OperationContext {
            schema: self.schema,
            operation: self.operation,
        };
        self.algorithm.to_dot_graph(
            |cost, is_in_steiner_tree| {
                Attrs::label_if(cost > 0, cost.to_string())
                    .bold()
                    .with_if(is_in_steiner_tree, "color=forestgreen,fontcolor=forestgreen")
                    .with_if(!is_in_steiner_tree, "color=royalblue,fontcolor=royalblue,style=dashed")
                    .to_string()
            },
            |node_id, is_in_steiner_tree| {
                self.query_solution_space
                    .graph
                    .node_weight(node_id)
                    .unwrap()
                    .pretty_label(self.query_solution_space, ctx)
                    .with_if(!is_in_steiner_tree, "style=dashed")
                    .with_if(is_in_steiner_tree, "color=forestgreen")
                    .to_string()
            },
        )
    }

    /// Use https://dreampuf.github.io/GraphvizOnline
    /// or `echo '..." | dot -Tsvg` from graphviz
    #[cfg(test)]
    pub fn to_dot_graph(&self) -> String {
        let ctx = OperationContext {
            schema: self.schema,
            operation: self.operation,
        };
        self.algorithm.to_dot_graph(
            |cost, is_in_steiner_tree| format!("cost={cost}, steiner={}", is_in_steiner_tree as usize),
            |node_id, is_in_steiner_tree| {
                Attrs::label(
                    self.query_solution_space
                        .graph
                        .node_weight(node_id)
                        .unwrap()
                        .label(self.query_solution_space, ctx),
                )
                .with(format!("steiner={}", is_in_steiner_tree as usize))
                .to_string()
            },
        )
    }
}

impl std::fmt::Debug for Solver<'_, '_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Solver").finish_non_exhaustive()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
struct ExtraRequiredNodeId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
struct ZeroCostParentEdgeId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
struct IncomingEdgeAndCostId(u32);

#[derive(Default, id_derives::IndexedFields)]
struct DispensableRequirementsMetadata {
    free_requirements: Vec<(NodeIndex, IdRange<ExtraRequiredNodeId>)>,
    maybe_costly_requirements: Vec<DispensableRequirements>,
    #[indexed_by(ExtraRequiredNodeId)]
    extra_required_nodes: Vec<NodeIndex>,
    #[indexed_by(ZeroCostParentEdgeId)]
    zero_cost_parent_edges: Vec<EdgeIndex>,
    #[indexed_by(IncomingEdgeAndCostId)]
    incoming_edges_and_cost: Vec<(EdgeIndex, Cost)>,
    independent_cost: Option<bool>,
}

#[derive(Clone, Copy)]
struct DispensableRequirements {
    zero_cost_parent_edge_ids: IdRange<ZeroCostParentEdgeId>,
    extra_required_node_ids: IdRange<ExtraRequiredNodeId>,
    incoming_edge_and_cost_ids: IdRange<IncomingEdgeAndCostId>,
}

impl DispensableRequirementsMetadata {
    fn extend_extra_required_nodes(
        &mut self,
        nodes: impl IntoIterator<Item = NodeIndex>,
    ) -> IdRange<ExtraRequiredNodeId> {
        let start = self.extra_required_nodes.len();
        self.extra_required_nodes.extend(nodes);
        IdRange::from(start..self.extra_required_nodes.len())
    }

    fn extend_zero_cost_parent_edges(
        &mut self,
        edges: impl IntoIterator<Item = EdgeIndex>,
    ) -> IdRange<ZeroCostParentEdgeId> {
        let start = self.zero_cost_parent_edges.len();
        self.zero_cost_parent_edges.extend(edges);
        IdRange::from(start..self.zero_cost_parent_edges.len())
    }

    fn extend_incoming_edges_and_cost(
        &mut self,
        edges_and_cost: impl IntoIterator<Item = (EdgeIndex, Cost)>,
    ) -> IdRange<IncomingEdgeAndCostId> {
        let start = self.incoming_edges_and_cost.len();
        self.incoming_edges_and_cost.extend(edges_and_cost);
        IdRange::from(start..self.incoming_edges_and_cost.len())
    }
}
