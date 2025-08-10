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

use crate::{Cost, SolutionSpaceGraph, SpaceEdge, SpaceNode};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct ExtraRequiredNodeId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct UnavoidableParentEdgeId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct IncomingEdgeAndCostId(u32);

#[derive(Default, id_derives::IndexedFields)]
pub(crate) struct DispensableRequirementsMetadata {
    pub free_requirements: Vec<(NodeIndex, IdRange<ExtraRequiredNodeId>)>,
    pub maybe_costly_requirements: Vec<DispensableRequirements>,
    #[indexed_by(ExtraRequiredNodeId)]
    extra_required_nodes: Vec<NodeIndex>,
    #[indexed_by(UnavoidableParentEdgeId)]
    unavoidable_parent_edges: Vec<EdgeIndex>,
    #[indexed_by(IncomingEdgeAndCostId)]
    incoming_edges_and_cost: Vec<(EdgeIndex, Cost)>,
}

#[derive(Clone, Copy)]
pub(crate) struct DispensableRequirements {
    pub unavoidable_parent_edge_ids: IdRange<UnavoidableParentEdgeId>,
    pub extra_required_node_ids: IdRange<ExtraRequiredNodeId>,
    pub incoming_edge_and_cost_ids: IdRange<IncomingEdgeAndCostId>,
}

impl DispensableRequirementsMetadata {
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
    pub fn ingest(&mut self, graph: &SolutionSpaceGraph<'_>) -> crate::Result<()> {
        struct IncomingEdgeWithDispensableRequirements {
            parent: NodeIndex,
            extra_required_node_ids: IdRange<ExtraRequiredNodeId>,
            incoming_edge_ix: EdgeIndex,
            edge_cost: Cost,
        }
        let mut buffer = Vec::with_capacity(graph.node_count() >> 4);

        // Used to intern required node id ranges
        let hasher = FxBuildHasher::default();
        let mut requirements_interner =
            hashbrown::HashTable::<IdRange<ExtraRequiredNodeId>>::with_capacity(graph.node_count() >> 4);

        for (node_ix, node) in graph.node_references() {
            if !matches!(node, SpaceNode::Resolver(_) | SpaceNode::ProvidableField(_)) {
                continue;
            }

            // Retrieve all the node ids on which we depend.
            let extra_required_node_ids = self.extend_extra_required_nodes(
                graph
                    .edges_directed(node_ix, Direction::Outgoing)
                    .filter(|edge| {
                        matches!(edge.weight(), SpaceEdge::Requires)
                            && graph[edge.target()]
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
            let key = &self[extra_required_node_ids];
            let extra_required_node_ids = match requirements_interner.entry(
                hasher.hash_one(key),
                |id| &self[*id] == key,
                |id| hasher.hash_one(&self[*id]),
            ) {
                Entry::Occupied(entry) => {
                    self.extra_required_nodes.truncate(extra_required_node_ids.start.into());

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
            if graph
                .edges_directed(node_ix, Direction::Incoming)
                .filter(|edge| matches!(edge.weight(), SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide))
                .all(|incoming_edge| {
                    let parent = incoming_edge.source();
                    self[extra_required_node_ids].iter().all(|required| {
                        graph
                            .edges_directed(parent, Direction::Outgoing)
                            .filter(|neighbor| matches!(neighbor.weight(), SpaceEdge::CanProvide))
                            .any(|neighbor| {
                                let mut found_requirement = false;
                                for edge in graph.edges_directed(neighbor.target(), Direction::Outgoing) {
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
                self.free_requirements.push((node_ix, extra_required_node_ids));
                continue;
            }

            for incoming_edge in graph.edges_directed(node_ix, Direction::Incoming) {
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
                .extend_incoming_edges_and_cost(chunk.into_iter().map(|item| (item.incoming_edge_ix, item.edge_cost)));

            // This will at least include the ProvidableField & Resolver that led to the
            // parent. As we'll necessarily take them for this particular edge, they'll be set
            // to 0 cost while estimating the requirement cost.
            let unavoidable_parent_edge_ids = self.extend_unavoidable_parent_edges(std::iter::from_fn(|| {
                let mut grand_parents = graph
                    .edges_directed(parent, Direction::Incoming)
                    .filter(|edge| matches!(edge.weight(), SpaceEdge::CreateChildResolver | SpaceEdge::CanProvide));

                let first = grand_parents.next()?;
                if grand_parents.next().is_none() {
                    parent = first.source();
                    Some(first.id())
                } else {
                    None
                }
            }));

            self.maybe_costly_requirements.push(DispensableRequirements {
                unavoidable_parent_edge_ids,
                extra_required_node_ids,
                incoming_edge_and_cost_ids,
            });
        }

        Ok(())
    }

    fn extend_extra_required_nodes(
        &mut self,
        nodes: impl IntoIterator<Item = NodeIndex>,
    ) -> IdRange<ExtraRequiredNodeId> {
        let start = self.extra_required_nodes.len();
        self.extra_required_nodes.extend(nodes);
        IdRange::from(start..self.extra_required_nodes.len())
    }

    fn extend_unavoidable_parent_edges(
        &mut self,
        edges: impl IntoIterator<Item = EdgeIndex>,
    ) -> IdRange<UnavoidableParentEdgeId> {
        let start = self.unavoidable_parent_edges.len();
        self.unavoidable_parent_edges.extend(edges);
        IdRange::from(start..self.unavoidable_parent_edges.len())
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
