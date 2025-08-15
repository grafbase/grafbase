mod builder;

use fxhash::FxHashMap;
use id_newtypes::IdRange;
use petgraph::{Graph, visit::GraphBase};

use crate::{QuerySolutionSpace, SpaceEdgeId, SpaceNodeId};

pub(crate) use builder::build_input_and_terminals;

pub(crate) type SteinerWeight = u16;
pub(crate) type SteinerGraph = Graph<(), SteinerWeight>;
pub(crate) type SteinerNodeId = <SteinerGraph as GraphBase>::NodeId;
pub(crate) type SteinerEdgeId = <SteinerGraph as GraphBase>::EdgeId;

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
    pub space_edge_id_to_edge_id: FxHashMap<SpaceEdgeId, SteinerEdgeId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct RequiredSteinerNodeId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct UnavoidableParentSteinerEdgeId(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, id_derives::Id)]
pub(crate) struct DependentSteinerEdgeWithInherentWeightId(u32);

// All NodeIndex & EdgeIndex are within the SteinerGraph.
#[derive(Default, id_derives::IndexedFields)]
pub(crate) struct DispensableRequirements {
    pub free: Vec<(SteinerNodeId, IdRange<RequiredSteinerNodeId>)>,
    pub groups: Vec<RequirementsGroup>,
    #[indexed_by(RequiredSteinerNodeId)]
    required_nodes: Vec<SteinerNodeId>,
    #[indexed_by(UnavoidableParentSteinerEdgeId)]
    unavoidable_parent_edges: Vec<SteinerEdgeId>,
    #[indexed_by(DependentSteinerEdgeWithInherentWeightId)]
    dependent_edges_with_inherent_weight: Vec<(SteinerEdgeId, SteinerWeight)>,
}

#[derive(Clone, Copy)]
pub(crate) struct RequirementsGroup {
    pub unavoidable_parent_edge_ids: IdRange<UnavoidableParentSteinerEdgeId>,
    pub required_node_ids: IdRange<RequiredSteinerNodeId>,
    pub dependent_edge_with_inherent_weight_ids: IdRange<DependentSteinerEdgeWithInherentWeightId>,
}

impl SteinerInput<'_> {
    pub(crate) fn to_space_node_id(&self, node_id: SteinerNodeId) -> SpaceNodeId {
        self.map.node_id_to_space_node_id[node_id.index()]
    }
}
