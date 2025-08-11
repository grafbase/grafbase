mod builder;

use id_newtypes::IdRange;

use crate::{
    Cost, QuerySolutionSpace, SpaceEdgeId, SpaceNodeId,
    solve::context::{SteinerEdgeId, SteinerGraph, SteinerNodeId},
};

pub(crate) struct SteinerInput<'schema> {
    pub space: QuerySolutionSpace<'schema>,
    pub graph: SteinerGraph,
    pub node_id_to_space_node_id: Vec<SpaceNodeId>,
    pub edge_id_to_space_edge_id: Vec<SpaceEdgeId>,
    pub space_node_id_to_node_id: Vec<SteinerNodeId>,
    pub requirements: DispensableRequirements,
}

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

impl<'schema> SteinerInput<'schema> {
    pub fn builder(space: QuerySolutionSpace<'schema>) -> Self {
        builder::SteinerInputBuilder::build(space)
    }
}
