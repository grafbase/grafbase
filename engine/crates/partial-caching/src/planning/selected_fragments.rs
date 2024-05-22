use cynic_parser::{
    executable::ids::{FragmentDefinitionId, SelectionId},
    ExecutableDocument,
};
use indexmap::{IndexMap, IndexSet};

use super::fragment_tracker::FragmentTracker;

#[derive(Default)]
pub struct FragmentSpreadSet {
    /// Fragments selected in an operation or fragment, and the
    /// selections that need to be included from that operation or fragment
    /// if the nested fragment needs to be included
    spreads: IndexMap<FragmentDefinitionId, IndexSet<SelectionId>>,
}

impl FragmentSpreadSet {
    pub fn spreads_for_fragment(&self, id: FragmentDefinitionId) -> Option<impl Iterator<Item = SelectionId> + '_> {
        Some(self.spreads.get(&id)?.iter().copied())
    }

    pub fn from_tracker(tracker: FragmentTracker, document: &ExecutableDocument) -> anyhow::Result<Self> {
        let mut this = Self::default();
        for (fragment_name, selections) in tracker.used_fragments {
            let fragment = document
                .fragments()
                .find(|fragment| fragment.name() == fragment_name)
                .ok_or_else(|| {
                    anyhow::anyhow!("The query contained a spread for a missing fragment: {fragment_name}")
                })?;

            this.spreads.insert(fragment.id(), selections);
        }

        Ok(this)
    }

    pub fn fragment_ids(&self) -> impl Iterator<Item = FragmentDefinitionId> + '_ {
        self.spreads.keys().copied()
    }
}
