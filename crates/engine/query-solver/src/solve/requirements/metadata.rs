use std::hash::BuildHasher as _;

use fxhash::FxBuildHasher;
use hashbrown::hash_table::Entry;
use id_newtypes::IdRange;
use itertools::Itertools as _;
use petgraph::{
    Direction,
    graph::{EdgeIndex, NodeIndex},
    visit::{EdgeRef as _, IntoNodeReferences as _},
};

use crate::{
    Cost, SolutionSpaceGraph, SpaceEdge, SpaceNode,
    solve::context::{SteinerContext, SteinerGraph},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct RequiredNodeId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct UnavoidableParentEdgeId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct DependentEdgeWithInherentCostId(u32);

// All NodeIndex & EdgeIndex are within the SteinerGraph.
#[derive(Default, id_derives::IndexedFields)]
pub(crate) struct DispensableRequirements {
    pub free_requirements: Vec<(NodeIndex, IdRange<RequiredNodeId>)>,
    pub groups: Vec<RequirementsGroup>,
    #[indexed_by(RequiredNodeId)]
    required_nodes: Vec<NodeIndex>,
    #[indexed_by(UnavoidableParentEdgeId)]
    unavoidable_parent_edges: Vec<EdgeIndex>,
    #[indexed_by(DependentEdgeWithInherentCostId)]
    dependent_edges_with_inherent_cost: Vec<(EdgeIndex, Cost)>,
}

#[derive(Clone, Copy)]
pub(crate) struct RequirementsGroup {
    pub unavoidable_parent_edge_ids: IdRange<UnavoidableParentEdgeId>,
    pub required_node_ids: IdRange<RequiredNodeId>,
    pub dependent_edge_with_inherent_cost_ids: IdRange<DependentEdgeWithInherentCostId>,
}

impl DispensableRequirements {
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
    pub fn ingest(&mut self, ctx: &SteinerContext<&SolutionSpaceGraph<'_>, SteinerGraph>) -> crate::Result<()> {
        struct DependentEdgeWithDispensableRequirements {
            dependent_edge_source: NodeIndex,
            dependent_edge_ix: EdgeIndex,
            inherent_cost: Cost,
            required_node_ids: IdRange<RequiredNodeId>,
        }
        let mut buffer = Vec::with_capacity(ctx.space_graph.node_count() >> 4);

        // Used to intern required node id ranges
        let hasher = FxBuildHasher::default();
        let mut requirements_interner =
            hashbrown::HashTable::<IdRange<RequiredNodeId>>::with_capacity(ctx.space_graph.node_count() >> 4);

        for (space_node_ix, space_node) in ctx.space_graph.node_references() {
            if !matches!(space_node, SpaceNode::Resolver(_) | SpaceNode::ProvidableField(_)) {
                continue;
            }

            // Retrieve all the node ids on which we depend.
            let required_node_ids = self.extend_extra_required_nodes(
                ctx.space_graph
                    .edges_directed(space_node_ix, Direction::Outgoing)
                    .filter(|edge| {
                        matches!(edge.weight(), SpaceEdge::Requires)
                            && ctx.space_graph[edge.target()]
                                .as_query_field()
                                .map(|field| !field.is_indispensable() && field.is_leaf())
                                .unwrap_or_default()
                    })
                    .map(|edge| ctx.to_node_ix(edge.target())),
            );
            if required_node_ids.is_empty() {
                continue;
            }

            // De-duplicate the requirements
            self[required_node_ids].sort_unstable();
            let key = &self[required_node_ids];
            let required_node_ids = match requirements_interner.entry(
                hasher.hash_one(key),
                |id| &self[*id] == key,
                |id| hasher.hash_one(&self[*id]),
            ) {
                Entry::Occupied(entry) => {
                    // Removing the requirements we just added, they exist already.
                    self.required_nodes.truncate(required_node_ids.start.into());
                    *entry.get()
                }
                Entry::Vacant(entry) => {
                    entry.insert(required_node_ids);
                    required_node_ids
                }
            };

            // Given a parent node, if there is a ProvidableField neighbor that provides our field
            // without any requirements, there is no cost associated with it.
            // If for each parent all the requirements have no cost, there is no extra cost at all
            // for this field.
            if ctx
                .space_graph
                .edges_directed(space_node_ix, Direction::Incoming)
                .filter(|edge| matches!(edge.weight(), SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide))
                .all(|incoming_edge| {
                    let parent = incoming_edge.source();
                    self[required_node_ids].iter().all(|required| {
                        ctx.space_graph
                            .edges_directed(parent, Direction::Outgoing)
                            .filter(|neighbor| matches!(neighbor.weight(), SpaceEdge::CanProvide))
                            .any(|neighbor| {
                                let mut found_requirement = false;
                                for edge in ctx.space_graph.edges_directed(neighbor.target(), Direction::Outgoing) {
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
                self.free_requirements
                    .push((ctx.to_node_ix(space_node_ix), required_node_ids));
                continue;
            }

            for dependent_space_edge in ctx.space_graph.edges_directed(space_node_ix, Direction::Incoming) {
                let inherent_cost = match dependent_space_edge.weight() {
                    SpaceEdge::CreateChildResolver => 1,
                    SpaceEdge::CanProvide => 0,
                    _ => continue,
                };
                buffer.push(DependentEdgeWithDispensableRequirements {
                    dependent_edge_source: ctx.to_node_ix(dependent_space_edge.source()),
                    required_node_ids,
                    inherent_cost,
                    dependent_edge_ix: ctx.to_edge_ix(dependent_space_edge.id()),
                });
            }
        }

        buffer.sort_unstable_by(|a, b| {
            a.dependent_edge_source
                .cmp(&b.dependent_edge_source)
                .then(a.required_node_ids.cmp(&b.required_node_ids))
        });

        for ((mut source, required_node_ids), chunk) in buffer
            .into_iter()
            .chunk_by(|item| (item.dependent_edge_source, item.required_node_ids))
            .into_iter()
        {
            let dependent_edge_with_inherent_cost_ids = self.extend_dependent_edges_with_inherent_cost(
                chunk
                    .into_iter()
                    .map(|item| (item.dependent_edge_ix, item.inherent_cost)),
            );

            // This will at least include the ProvidableField & Resolver that led to the
            // parent. As we'll necessarily take them for this particular edge, they'll be set
            // to 0 cost while estimating the requirement cost.
            let unavoidable_parent_edge_ids = self.extend_unavoidable_parent_edges(std::iter::from_fn(|| {
                let mut grand_parents = ctx.graph.edges_directed(source, Direction::Incoming);

                let first = grand_parents.next()?;
                if grand_parents.next().is_none() {
                    source = first.source();
                    Some(first.id())
                } else {
                    None
                }
            }));

            self.groups.push(RequirementsGroup {
                unavoidable_parent_edge_ids,
                required_node_ids,
                dependent_edge_with_inherent_cost_ids,
            });
        }

        Ok(())
    }

    fn extend_extra_required_nodes(&mut self, nodes: impl IntoIterator<Item = NodeIndex>) -> IdRange<RequiredNodeId> {
        let start = self.required_nodes.len();
        self.required_nodes.extend(nodes);
        IdRange::from(start..self.required_nodes.len())
    }

    fn extend_unavoidable_parent_edges(
        &mut self,
        edges: impl IntoIterator<Item = EdgeIndex>,
    ) -> IdRange<UnavoidableParentEdgeId> {
        let start = self.unavoidable_parent_edges.len();
        self.unavoidable_parent_edges.extend(edges);
        IdRange::from(start..self.unavoidable_parent_edges.len())
    }

    fn extend_dependent_edges_with_inherent_cost(
        &mut self,
        edge_costs: impl IntoIterator<Item = (EdgeIndex, Cost)>,
    ) -> IdRange<DependentEdgeWithInherentCostId> {
        let start = self.dependent_edges_with_inherent_cost.len();
        self.dependent_edges_with_inherent_cost.extend(edge_costs);
        IdRange::from(start..self.dependent_edges_with_inherent_cost.len())
    }
}
