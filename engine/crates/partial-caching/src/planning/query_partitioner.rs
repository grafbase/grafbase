use cynic_parser::executable::{
    ids::{FragmentDefinitionId, SelectionId},
    Selection,
};
use indexmap::IndexMap;
use registry_for_cache::CacheControl;

use super::visitor::FieldEdge;
use crate::query_subset::CacheGroup;

/// A visitor that groups fields by their caching rules
pub(crate) struct QueryPartitioner {
    /// A stack of selections of the current traversal
    selection_stack: Vec<SelectionId>,

    cache_control_stack: Vec<CacheControl>,

    current_fragment: Option<FragmentDefinitionId>,

    pub cache_partitions: IndexMap<CacheControl, CacheGroup>,
    pub nocache_partition: CacheGroup,
}

impl QueryPartitioner {
    pub fn new(root_cache_control: Option<&CacheControl>) -> Self {
        QueryPartitioner {
            selection_stack: vec![],
            current_fragment: None,
            cache_partitions: IndexMap::new(),
            nocache_partition: CacheGroup::default(),
            cache_control_stack: root_cache_control.into_iter().cloned().collect::<Vec<_>>(),
        }
    }

    pub fn for_next_fragment(
        mut self,
        next_fragment: FragmentDefinitionId,
        starting_cache_control: Option<&CacheControl>,
    ) -> Self {
        // If this isn't empty something has gone horribly wrong
        assert!(self.selection_stack.is_empty());

        self.cache_control_stack.clear();
        self.cache_control_stack.extend(starting_cache_control.cloned());

        QueryPartitioner {
            current_fragment: Some(next_fragment),
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
        if let Some(cache_control) = edge.cache_control() {
            self.cache_control_stack.push((*cache_control).clone());
        }

        if edge.selection.selection_set().len() != 0 {
            // If this field has child selections then its not a leaf,
            // and the partitioner only acts on leaves, so just return.
            return;
        }

        match self.cache_control_stack.last() {
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

    fn exit_field(&mut self, edge: FieldEdge<'_>) {
        if edge.cache_control().is_some() {
            self.cache_control_stack.pop();
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
