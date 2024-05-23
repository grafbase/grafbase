use cynic_parser::executable::{
    ids::{FragmentDefinitionId, SelectionId},
    Selection,
};
use indexmap::IndexMap;

use super::visitor::FieldEdge;
use crate::query_subset::CacheGroup;

/// A visitor that groups fields by their caching rules
pub(crate) struct QueryPartitioner {
    /// A stack of selections the current traversal
    selection_stack: Vec<SelectionId>,

    current_fragment: Option<FragmentDefinitionId>,

    pub cache_partitions: IndexMap<registry_for_cache::CacheControl, CacheGroup>,
    pub nocache_partition: CacheGroup,
}

impl QueryPartitioner {
    pub fn new() -> Self {
        QueryPartitioner {
            selection_stack: vec![],
            current_fragment: None,
            cache_partitions: IndexMap::new(),
            nocache_partition: CacheGroup::default(),
        }
    }

    pub fn for_next_fragment(self, current_fragment: FragmentDefinitionId) -> Self {
        // If this isn't empty something has gone horribly wrong
        assert!(self.selection_stack.is_empty());

        QueryPartitioner {
            current_fragment: Some(current_fragment),
            ..self
        }
    }
}

impl super::visitor::Visitor for QueryPartitioner {
    fn enter_selection(&mut self, id: SelectionId, _selection: Selection<'_>) {
        self.selection_stack.push(id)
    }

    fn exit_selection(&mut self, _id: SelectionId, _selection: Selection<'_>) {
        self.selection_stack.pop();
    }

    fn enter_field(&mut self, edge: FieldEdge<'_>) {
        if edge.selection.selection_set().len() != 0 {
            // We're only concerned with the cache control settings of leaf fields
            // at the moment
            return;
        }

        match edge.field.and_then(|field| field.cache_control()) {
            Some(cache_control) => self
                .cache_partitions
                .entry(cache_control.clone())
                .or_default()
                .update(&self.selection_stack, self.current_fragment),

            None => self
                .nocache_partition
                .update(&self.selection_stack, self.current_fragment),
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
