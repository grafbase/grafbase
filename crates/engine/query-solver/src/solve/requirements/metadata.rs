use std::hash::BuildHasher as _;

use fxhash::FxBuildHasher;
use hashbrown::hash_table::Entry;
use id_newtypes::IdRange;
use itertools::Itertools as _;
use petgraph::{
    Direction,
    visit::{EdgeRef as _, IntoNodeReferences as _},
};

use crate::{
    Cost, SolutionSpaceGraph, SpaceEdge, SpaceEdgeId, SpaceNode, SpaceNodeId,
    solve::{
        context::{SteinerContext, SteinerEdgeId, SteinerGraph, SteinerNodeId},
        input::SteinerGraphBuilder,
    },
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct RequiredSteinerNodeId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct UnavoidableParentSteinerEdgeId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct DependentSteinerEdgeWithInherentCostId(u32);

// All NodeIndex & EdgeIndex are within the SteinerGraph.
#[derive(Default, id_derives::IndexedFields)]
pub(crate) struct DispensableRequirements {
    pub free_requirements: Vec<(SteinerNodeId, IdRange<RequiredSteinerNodeId>)>,
    pub groups: Vec<RequirementsGroup>,
    #[indexed_by(RequiredSteinerNodeId)]
    required_nodes: Vec<SteinerNodeId>,
    #[indexed_by(UnavoidableParentSteinerEdgeId)]
    unavoidable_parent_edges: Vec<SteinerEdgeId>,
    #[indexed_by(DependentSteinerEdgeWithInherentCostId)]
    dependent_edges_with_inherent_cost: Vec<(SteinerEdgeId, Cost)>,
}

#[derive(Clone, Copy)]
pub(crate) struct RequirementsGroup {
    pub unavoidable_parent_edge_ids: IdRange<UnavoidableParentSteinerEdgeId>,
    pub required_node_ids: IdRange<RequiredSteinerNodeId>,
    pub dependent_edge_with_inherent_cost_ids: IdRange<DependentSteinerEdgeWithInherentCostId>,
}

pub(crate) struct DispensableRequirementsBuilder {
    pub out: DispensableRequirements,
    buffer: Vec<DependentEdgeWithDispensableRequirements>,
    space_node_buffer: Vec<SpaceNodeId>,
    requirement_hasher: FxBuildHasher,
    requirement_interner: hashbrown::HashTable<IdRange<RequiredSteinerNodeId>>,
    tmp_space_required_node_ids: Vec<SpaceNodeId>,
}

struct DependentEdgeWithDispensableRequirements {
    dependent_space_edge_source: SpaceNodeId,
    dependent_space_edge_id: SpaceEdgeId,
    inherent_cost: Cost,
    required_node_ids: IdRange<RequiredSteinerNodeId>,
}

pub struct DetectedRequirements<'s> {
    builder: &'s mut DispensableRequirementsBuilder,
    space_node_id: SpaceNodeId,
    ids: IdRange<RequiredSteinerNodeId>,
}

impl<'s> DetectedRequirements<'s> {
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn forget_because_indispensable(self) -> (&'s [SpaceNodeId], &'s [SteinerNodeId]) {
        let Self { builder, ids, .. } = self;
        (&builder.tmp_space_required_node_ids, &builder.out[ids])
    }

    pub fn ingest_as_dispensable(
        self,
        space_graph: &SolutionSpaceGraph<'_>,
        node_id: SteinerNodeId,
    ) -> (&'s [SpaceNodeId], &'s [SteinerNodeId]) {
        let Self {
            builder,
            ids,
            space_node_id,
        } = self;
        builder.ingest(space_graph, node_id, space_node_id, ids)
    }
}

impl DispensableRequirementsBuilder {
    pub fn new(space_graph: &SolutionSpaceGraph<'_>) -> Self {
        Self {
            buffer: Vec::with_capacity(space_graph.node_count() >> 4),
            space_node_buffer: Vec::new(),
            requirement_hasher: FxBuildHasher::default(),
            requirement_interner: hashbrown::HashTable::with_capacity(space_graph.node_count() >> 4),
            tmp_space_required_node_ids: Vec::new(),
            out: DispensableRequirements::default(),
        }
    }

    pub fn collect<'s>(
        &'s mut self,
        space_graph: &SolutionSpaceGraph<'_>,
        builder: &mut SteinerGraphBuilder,
        space_node_id: SpaceNodeId,
    ) -> DetectedRequirements<'s> {
        // Retrieve all the node ids on which we depend.
        self.tmp_space_required_node_ids.clear();
        let ids = self.out.extend_extra_required_nodes(
            space_graph
                .edges_directed(space_node_id, Direction::Outgoing)
                .filter(|edge| {
                    matches!(edge.weight(), SpaceEdge::Requires)
                        && space_graph[edge.target()]
                            .as_query_field()
                            .map(|field| !field.is_indispensable() && field.is_leaf())
                            .unwrap_or_default()
                })
                .map(|edge| {
                    let required_space_node_id = edge.target();
                    self.tmp_space_required_node_ids.push(required_space_node_id);
                    builder.get_or_insert_node(required_space_node_id)
                }),
        );
        DetectedRequirements {
            builder: self,
            space_node_id,
            ids,
        }
    }

    fn ingest<'s>(
        &'s mut self,
        space_graph: &SolutionSpaceGraph<'_>,
        node_id: SteinerNodeId,
        space_node_id: SpaceNodeId,
        required_node_ids: IdRange<RequiredSteinerNodeId>,
    ) -> (&'s [SpaceNodeId], &'s [SteinerNodeId]) {
        if required_node_ids.is_empty() {
            return (&[], &[]);
        }

        // De-duplicate the requirements
        self.out[required_node_ids].sort_unstable();
        let key = &self.out[required_node_ids];
        let required_node_ids = match self.requirement_interner.entry(
            self.requirement_hasher.hash_one(key),
            |id| &self.out[*id] == key,
            |id| self.requirement_hasher.hash_one(&self.out[*id]),
        ) {
            Entry::Occupied(entry) => {
                // Removing the requirements we just added, they exist already.
                self.out.required_nodes.truncate(required_node_ids.start.into());
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
        if space_graph
            .edges_directed(space_node_id, Direction::Incoming)
            .filter(|edge| matches!(edge.weight(), SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide))
            .all(|incoming_edge| {
                let parent = incoming_edge.source();
                self.tmp_space_required_node_ids.iter().all(|required| {
                    space_graph
                        .edges_directed(parent, Direction::Outgoing)
                        .filter(|neighbor| matches!(neighbor.weight(), SpaceEdge::CanProvide))
                        .any(|neighbor| {
                            let mut found_requirement = false;
                            for edge in space_graph.edges_directed(neighbor.target(), Direction::Outgoing) {
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
            self.out.free_requirements.push((node_id, required_node_ids));
        } else {
            for dependent_space_edge in space_graph.edges_directed(space_node_id, Direction::Incoming) {
                let inherent_cost = match dependent_space_edge.weight() {
                    SpaceEdge::CreateChildResolver => 1,
                    SpaceEdge::CanProvide => 0,
                    _ => continue,
                };
                self.buffer.push(DependentEdgeWithDispensableRequirements {
                    dependent_space_edge_source: dependent_space_edge.source(),
                    required_node_ids,
                    inherent_cost,
                    dependent_space_edge_id: dependent_space_edge.id(),
                });
            }
        }

        (&self.tmp_space_required_node_ids, &self.out[required_node_ids])
    }

    pub fn build(self) -> DispensableRequirements {
        todo!()
    }
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
            dependent_edge_source: SteinerNodeId,
            dependent_edge_ix: SteinerEdgeId,
            inherent_cost: Cost,
            required_node_ids: IdRange<RequiredSteinerNodeId>,
        }
        let mut buffer = Vec::with_capacity(ctx.space_graph.node_count() >> 4);

        // Used to intern required node id ranges
        let hasher = FxBuildHasher::default();
        let mut requirements_interner =
            hashbrown::HashTable::<IdRange<RequiredSteinerNodeId>>::with_capacity(ctx.space_graph.node_count() >> 4);
        let mut space_required_node_ids = Vec::new();

        for (space_node_ix, space_node) in ctx.space_graph.node_references() {
            if !matches!(space_node, SpaceNode::Resolver(_) | SpaceNode::ProvidableField(_)) {
                continue;
            }

            // Retrieve all the node ids on which we depend.
            space_required_node_ids.clear();
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
                    .map(|edge| {
                        space_required_node_ids.push(edge.target());
                        ctx.to_node_ix(edge.target())
                    }),
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
                    space_required_node_ids.iter().all(|required| {
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

    fn extend_extra_required_nodes(
        &mut self,
        nodes: impl IntoIterator<Item = SteinerNodeId>,
    ) -> IdRange<RequiredSteinerNodeId> {
        let start = self.required_nodes.len();
        self.required_nodes.extend(nodes);
        IdRange::from(start..self.required_nodes.len())
    }

    fn extend_unavoidable_parent_edges(
        &mut self,
        edges: impl IntoIterator<Item = SteinerEdgeId>,
    ) -> IdRange<UnavoidableParentSteinerEdgeId> {
        let start = self.unavoidable_parent_edges.len();
        self.unavoidable_parent_edges.extend(edges);
        IdRange::from(start..self.unavoidable_parent_edges.len())
    }

    fn extend_dependent_edges_with_inherent_cost(
        &mut self,
        edge_costs: impl IntoIterator<Item = (SteinerEdgeId, Cost)>,
    ) -> IdRange<DependentSteinerEdgeWithInherentCostId> {
        let start = self.dependent_edges_with_inherent_cost.len();
        self.dependent_edges_with_inherent_cost.extend(edge_costs);
        IdRange::from(start..self.dependent_edges_with_inherent_cost.len())
    }
}
