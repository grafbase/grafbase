mod ancestry;
mod graph;
mod spread_set;
mod tracker;

use cynic_parser::executable::ids::FragmentDefinitionId;
use registry_for_cache::CacheControl;

pub(super) use self::{
    ancestry::{calculate_ancestry, FragmentAncestry},
    spread_set::FragmentSpreadSet,
    tracker::FragmentTracker,
};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct FragmentKey {
    pub id: FragmentDefinitionId,

    // The cache control that was active when this fragment was spread
    pub spread_cache_control: Option<CacheControl>,
}

impl FragmentKey {
    pub fn new(id: FragmentDefinitionId, spread_cache_control: Option<CacheControl>) -> Self {
        Self {
            id,
            spread_cache_control,
        }
    }
}
