use std::hash::BuildHasher as _;

use fxhash::FxBuildHasher;
use hashbrown::hash_table::Entry;
use id_newtypes::IdRange;
use itertools::Itertools as _;
use petgraph::{Direction, visit::EdgeRef as _};

use crate::{
    SolutionSpaceGraph, SpaceEdge, SpaceEdgeId, SpaceNodeId,
    solve::{
        input::{
            DependentSteinerEdgeWithInherentWeightId, DispensableRequirements, FreeRequirement, RequiredSpaceNodeId,
            RequiredSteinerNodeId, RequirementsGroup, SteinerEdgeId, SteinerInputMap, SteinerNodeId, SteinerWeight,
            UnavoidableParentSteinerEdgeId, builder::SteinerInputBuilder,
        },
        steiner_tree::SteinerTree,
    },
};

#[derive(id_derives::IndexedFields)]
pub(crate) struct DispensableRequirementsBuilder {
    free: Vec<(SteinerNodeId, IdRange<RequiredSpaceNodeId>)>,
    dispensable: Vec<DependentEdgeWithDispensableRequirements>,
    hasher: FxBuildHasher,
    interner: hashbrown::HashTable<IdRange<RequiredSpaceNodeId>>,
    #[indexed_by(RequiredSpaceNodeId)]
    required_space_nodes: Vec<SpaceNodeId>,
}

struct DependentEdgeWithDispensableRequirements {
    dependent_space_edge_source: SpaceNodeId,
    dependent_space_edge_id: SpaceEdgeId,
    required_space_node_ids: IdRange<RequiredSpaceNodeId>,
}

pub struct DetectedRequirements<'s> {
    builder: &'s mut DispensableRequirementsBuilder,
    space_node_id: SpaceNodeId,
    required_space_node_ids: IdRange<RequiredSpaceNodeId>,
}

impl<'s> DetectedRequirements<'s> {
    pub fn is_empty(&self) -> bool {
        self.required_space_node_ids.is_empty()
    }

    pub fn forget_because_indispensable(self, f: impl FnOnce(&[SpaceNodeId])) {
        let Self {
            builder,
            required_space_node_ids: ids,
            ..
        } = self;
        f(&builder[ids]);
        builder.required_space_nodes.truncate(ids.start.into());
    }

    pub fn ingest_as_dispensable(
        self,
        space_graph: &SolutionSpaceGraph<'_>,
        node_id: SteinerNodeId,
    ) -> &'s [SpaceNodeId] {
        let Self {
            builder,
            required_space_node_ids: ids,
            space_node_id,
        } = self;
        builder.ingest(space_graph, node_id, space_node_id, ids)
    }
}

impl DispensableRequirementsBuilder {
    pub fn new(space_graph: &SolutionSpaceGraph<'_>) -> Self {
        let n = space_graph.node_count() >> 5;
        Self {
            free: Vec::with_capacity(n),
            dispensable: Vec::with_capacity(n),
            hasher: FxBuildHasher::default(),
            interner: hashbrown::HashTable::with_capacity(n),
            required_space_nodes: Vec::with_capacity(n),
        }
    }

    pub fn collect<'s>(
        &'s mut self,
        space_graph: &SolutionSpaceGraph<'_>,
        space_node_id: SpaceNodeId,
    ) -> DetectedRequirements<'s> {
        // Retrieve all the node ids on which we depend.
        let ids = self.extend_extra_required_space_nodes(
            space_graph
                .edges_directed(space_node_id, Direction::Outgoing)
                .filter(|edge| {
                    matches!(edge.weight(), SpaceEdge::Requires)
                        && space_graph[edge.target()]
                            .as_query_field()
                            .map(|field| !field.is_indispensable() && field.is_leaf())
                            .unwrap_or_default()
                })
                .map(|edge| edge.target()),
        );
        DetectedRequirements {
            builder: self,
            space_node_id,
            required_space_node_ids: ids,
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
    fn ingest(
        &mut self,
        space_graph: &SolutionSpaceGraph<'_>,
        node_id: SteinerNodeId,
        space_node_id: SpaceNodeId,
        required_space_node_ids: IdRange<RequiredSpaceNodeId>,
    ) -> &[SpaceNodeId] {
        if required_space_node_ids.is_empty() {
            return &[];
        }

        // De-duplicate the requirements
        self[required_space_node_ids].sort_unstable();
        let key = &self.required_space_nodes[required_space_node_ids.as_usize()];
        let required_space_node_ids = match self.interner.entry(
            self.hasher.hash_one(key),
            |ids| &self.required_space_nodes[ids.as_usize()] == key,
            |ids| self.hasher.hash_one(&self.required_space_nodes[ids.as_usize()]),
        ) {
            Entry::Occupied(entry) => {
                // Removing the requirements we just added, they exist already.
                self.required_space_nodes.truncate(required_space_node_ids.start.into());
                *entry.get()
            }
            Entry::Vacant(entry) => {
                entry.insert(required_space_node_ids);
                required_space_node_ids
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
                self[required_space_node_ids].iter().all(|required| {
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
            self.free.push((node_id, required_space_node_ids));
        } else {
            for dependent_space_edge in space_graph
                .edges_directed(space_node_id, Direction::Incoming)
                .filter(|edge| matches!(edge.weight(), SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide))
            {
                self.dispensable.push(DependentEdgeWithDispensableRequirements {
                    dependent_space_edge_source: dependent_space_edge.source(),
                    required_space_node_ids,
                    dependent_space_edge_id: dependent_space_edge.id(),
                });
            }
        }

        &self[required_space_node_ids]
    }

    fn extend_extra_required_space_nodes(
        &mut self,
        nodes: impl IntoIterator<Item = SpaceNodeId>,
    ) -> IdRange<RequiredSpaceNodeId> {
        let start = self.required_space_nodes.len();
        self.required_space_nodes.extend(nodes);
        IdRange::from(start..self.required_space_nodes.len())
    }

    pub fn build(
        mut self,
        builder: &SteinerInputBuilder<'_, '_, '_>,
        steiner_tree: &SteinerTree,
    ) -> DispensableRequirements {
        let mut out = DispensableRequirements {
            free: Vec::with_capacity(self.free.len()),
            groups: Vec::with_capacity(self.dispensable.len()),
            required_space_nodes: std::mem::take(&mut self.required_space_nodes),
            required_nodes: Vec::with_capacity(self.required_space_nodes.len()),
            unavoidable_parent_edges: Vec::with_capacity(self.dispensable.len()),
            dependent_edges_with_inherent_weight: Vec::with_capacity(self.dispensable.len()),
        };

        let mut buffer = Vec::new();
        for (node_id, required_space_node_ids) in std::mem::take(&mut self.free) {
            let required_node_ids =
                out.extend_extra_required_nodes(&builder.map, steiner_tree, required_space_node_ids, &mut buffer);

            out.free.push(FreeRequirement {
                node_id,
                required_node_ids,
                required_space_node_ids,
            })
        }

        self.dispensable.sort_unstable_by(|a, b| {
            a.dependent_space_edge_source
                .cmp(&b.dependent_space_edge_source)
                .then(a.required_space_node_ids.cmp(&b.required_space_node_ids))
        });
        for ((space_edge_source_id, required_space_node_ids), chunk) in std::mem::take(&mut self.dispensable)
            .into_iter()
            .chunk_by(|item| (item.dependent_space_edge_source, item.required_space_node_ids))
            .into_iter()
        {
            let required_node_ids =
                out.extend_extra_required_nodes(&builder.map, steiner_tree, required_space_node_ids, &mut buffer);

            let dependent_edge_with_inherent_weight_ids =
                out.extend_dependent_edges_with_inherent_weight(chunk.into_iter().map(|item| {
                    let Some(edge_id) = builder
                        .map
                        .space_edge_id_to_edge_id
                        .get(&item.dependent_space_edge_id)
                        .copied()
                    else {
                        let (src, dst) = builder
                            .space
                            .graph
                            .edge_endpoints(item.dependent_space_edge_id)
                            .unwrap();
                        unreachable!(
                            "The space edge should have been added to the builder: {} -> {}",
                            builder.space.graph[src].label(builder.space, builder.ctx),
                            builder.space.graph[dst].label(builder.space, builder.ctx)
                        );
                    };
                    (edge_id, builder.graph[edge_id])
                }));

            let mut source_id = builder.map.space_node_id_to_node_id[space_edge_source_id.index()];
            // This will at least include the ProvidableField & Resolver that led to the
            // parent. As we'll necessarily take them for this particular edge, they'll be set
            // to 0 cost while estimating the requirement cost.
            let unavoidable_parent_edge_ids = out.extend_unavoidable_parent_edges(std::iter::from_fn(|| {
                let mut grand_parents = builder.graph.edges_directed(source_id, Direction::Incoming);

                let first = grand_parents.next()?;
                if grand_parents.next().is_none() {
                    source_id = first.source();
                    Some(first.id())
                } else {
                    None
                }
            }));

            out.groups.push(RequirementsGroup {
                unavoidable_parent_edge_ids,
                required_space_node_ids,
                required_node_ids,
                dependent_edge_with_inherent_weight_ids,
            });
        }

        out
    }
}

impl DispensableRequirements {
    fn extend_unavoidable_parent_edges(
        &mut self,
        edges: impl IntoIterator<Item = SteinerEdgeId>,
    ) -> IdRange<UnavoidableParentSteinerEdgeId> {
        let start = self.unavoidable_parent_edges.len();
        self.unavoidable_parent_edges.extend(edges);
        IdRange::from(start..self.unavoidable_parent_edges.len())
    }

    fn extend_dependent_edges_with_inherent_weight(
        &mut self,
        edge_weights: impl IntoIterator<Item = (SteinerEdgeId, SteinerWeight)>,
    ) -> IdRange<DependentSteinerEdgeWithInherentWeightId> {
        let start = self.dependent_edges_with_inherent_weight.len();
        self.dependent_edges_with_inherent_weight.extend(edge_weights);
        IdRange::from(start..self.dependent_edges_with_inherent_weight.len())
    }

    fn extend_extra_required_nodes(
        &mut self,
        map: &SteinerInputMap,
        steiner_tree: &SteinerTree,
        required_space_node_ids: IdRange<RequiredSpaceNodeId>,
        buffer: &mut Vec<SteinerNodeId>,
    ) -> IdRange<RequiredSteinerNodeId> {
        buffer.extend(
            self.required_space_nodes[required_space_node_ids.as_usize()]
                .iter()
                .map(|id| map.space_node_id_to_node_id[id.index()])
                // If already a terminal, the requirement doesn't matter.
                .filter(|&id| !steiner_tree.is_terminal[id.index()]),
        );
        buffer.sort_unstable();
        buffer.dedup();

        let start = self.required_nodes.len();
        self.required_nodes.append(buffer);
        IdRange::from(start..self.required_nodes.len())
    }
}
