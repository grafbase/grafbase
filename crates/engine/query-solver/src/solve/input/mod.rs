mod builder;

use id_newtypes::IdRange;

use crate::{
    Cost, QuerySolutionSpace, SpaceEdgeId, SpaceNodeId,
    solve::context::{SteinerEdgeId, SteinerGraph, SteinerNodeId},
};

pub(crate) struct SteinerInput<'schema> {
    pub space: QuerySolutionSpace<'schema>,
    pub graph: SteinerGraph,
    pub root_node_id: SteinerNodeId,
    pub map: InputMap,
    pub requirements: DispensableRequirements,
}

pub(crate) struct InputMap {
    pub node_id_to_space_node_id: Vec<SpaceNodeId>,
    pub edge_id_to_space_edge_id: Vec<SpaceEdgeId>,
    pub space_node_id_to_node_id: Vec<SteinerNodeId>,
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
    pub free: Vec<(SteinerNodeId, IdRange<RequiredSteinerNodeId>)>,
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

pub fn build_input_and_terminals<'schema>(
    space: QuerySolutionSpace<'schema>,
) -> (SteinerInput<'schema>, Vec<SteinerNodeId>) {
    builder::SteinerInputBuilder::build(space)
}

impl SteinerInput<'_> {
    pub(crate) fn to_node_id(&self, space_node_id: SpaceNodeId) -> Option<SteinerNodeId> {
        let id = self.map.space_node_id_to_node_id[space_node_id.index()];
        if id.index() as u32 == u32::MAX {
            return None;
        }
        Some(id)
    }

    pub(crate) fn to_space_node_id(&self, node_id: SteinerNodeId) -> SpaceNodeId {
        self.map.node_id_to_space_node_id[node_id.index()]
    }
}
