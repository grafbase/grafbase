use cynic_parser::executable::ids::SelectionId;
use indexmap::{IndexMap, IndexSet};

use super::FragmentKey;

#[derive(Default)]
pub struct FragmentSpreadSet {
    /// Fragments selected in an operation or fragment, and the
    /// selections that need to be included from that operation or fragment
    /// if the nested fragment needs to be included
    spreads: IndexMap<FragmentKey, IndexSet<SelectionId>>,
}

impl FragmentSpreadSet {
    pub fn insert(&mut self, key: FragmentKey, selections: IndexSet<SelectionId>) {
        self.spreads.entry(key).or_default().extend(selections)
    }

    pub fn spreads_for_fragment(&self, id: &FragmentKey) -> Option<impl Iterator<Item = SelectionId> + '_> {
        Some(self.spreads.get(id)?.iter().copied())
    }

    pub fn fragment_keys(&self) -> impl Iterator<Item = FragmentKey> + '_ {
        self.spreads.keys().cloned()
    }
}
