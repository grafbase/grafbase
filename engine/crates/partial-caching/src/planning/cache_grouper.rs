use cynic_parser::executable::{
    ids::{FragmentDefinitionId, SelectionId},
    Selection,
};
use indexmap::IndexMap;

use super::visitor::FieldEdge;
use crate::query_subset::CacheGroup;

/// A visitor that groups fields by their caching rules
pub(crate) struct CacheGrouper {
    /// A stack of selections the current traversal
    selection_stack: Vec<SelectionId>,

    current_fragment: Option<FragmentDefinitionId>,

    pub cache_groups: IndexMap<registry_for_cache::CacheControl, CacheGroup>,
    pub uncached_group: CacheGroup,
}

impl CacheGrouper {
    pub fn new() -> Self {
        CacheGrouper {
            selection_stack: vec![],
            current_fragment: None,
            cache_groups: IndexMap::new(),
            uncached_group: CacheGroup::default(),
        }
    }

    pub fn with_current_fragment(self, current_fragment: FragmentDefinitionId) -> Self {
        // If this isn't empty something has gone horribly wrong
        assert!(self.selection_stack.is_empty());

        CacheGrouper {
            current_fragment: Some(current_fragment),
            ..self
        }
    }
}

impl super::visitor::Visitor for CacheGrouper {
    fn enter_selection(&mut self, id: SelectionId, _selection: Selection<'_>) {
        self.selection_stack.push(id)
    }

    fn exit_selection(&mut self, _id: SelectionId, _selection: Selection<'_>) {
        self.selection_stack.pop();
    }

    fn enter_field(&mut self, edge: FieldEdge<'_>) {
        if edge.selection.selection_set().len() != 0 {
            // We're only concerned with the cache control of leaf fields
            return;
        }

        match edge.field.and_then(|field| field.cache_control()) {
            Some(cache_control) => self
                .cache_groups
                .entry(cache_control.clone())
                .or_default()
                .update(&self.selection_stack, self.current_fragment),

            None => self.uncached_group.update(&self.selection_stack, self.current_fragment),
        }
    }
}

impl CacheGroup {
    fn update(&mut self, selection_stack: &[SelectionId], current_fragment: Option<FragmentDefinitionId>) {
        self.fragments.extend(current_fragment);
        for id in selection_stack.iter().rev() {
            if self.selections.contains(id) {
                // If the current node is already in the set we can stop
                // walking up the tree
                return;
            }
            self.selections.insert(*id);
        }
    }
}
