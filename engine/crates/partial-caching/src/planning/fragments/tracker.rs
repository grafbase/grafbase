use cynic_parser::executable::{ids::SelectionId, FragmentSpread, Selection};
use registry_for_cache::CacheControl;

use crate::planning::visitor::FieldEdge;

use super::{FragmentKey, FragmentSpreadSet};

/// A visitor that tracks used fragments in a query, and which selections are
/// ancestors of spreads of those fragments
pub struct FragmentTracker {
    cache_control_stack: Vec<CacheControl>,

    selection_stack: Vec<SelectionId>,

    spreads: FragmentSpreadSet,
    missing_fragments: Vec<String>,
}

impl FragmentTracker {
    pub fn new(root_cache_control: Option<&CacheControl>) -> Self {
        FragmentTracker {
            selection_stack: vec![],
            cache_control_stack: root_cache_control.into_iter().cloned().collect::<Vec<_>>(),
            spreads: FragmentSpreadSet::default(),
            missing_fragments: vec![],
        }
    }

    fn key_for(&self, spread: FragmentSpread<'_>) -> Option<FragmentKey> {
        Some(FragmentKey::new(
            spread.fragment()?.id(),
            self.cache_control_stack.last().cloned(),
        ))
    }

    pub fn into_spreads(self) -> anyhow::Result<FragmentSpreadSet> {
        if !self.missing_fragments.is_empty() {
            if self.missing_fragments.len() == 1 {
                return Err(anyhow::anyhow!(
                    "Could not find a fragment named {}",
                    self.missing_fragments[0]
                ));
            } else {
                return Err(anyhow::anyhow!(
                    "Could not find fragments named: {}",
                    self.missing_fragments.join(", ")
                ));
            }
        }
        Ok(self.spreads)
    }
}

impl super::super::visitor::Visitor for FragmentTracker {
    fn enter_selection(&mut self, id: SelectionId, _selection: Selection<'_>) {
        self.selection_stack.push(id)
    }

    fn exit_selection(&mut self, _id: SelectionId, _selection: Selection<'_>) {
        self.selection_stack.pop();
    }

    fn enter_field(&mut self, edge: FieldEdge<'_>) {
        if let Some(cache_control) = edge.cache_control() {
            self.cache_control_stack.push((*cache_control).clone());
        }
    }

    fn exit_field(&mut self, edge: FieldEdge<'_>) {
        if edge.cache_control().is_some() {
            self.cache_control_stack.pop();
        }
    }

    fn fragment_spread(&mut self, spread: FragmentSpread<'_>) {
        let Some(key) = self.key_for(spread) else {
            self.missing_fragments.push(spread.fragment_name().to_string());
            return;
        };

        self.spreads.insert(key, self.selection_stack.iter().copied().collect())
    }
}
