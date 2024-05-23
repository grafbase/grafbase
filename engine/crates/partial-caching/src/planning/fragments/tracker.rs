use cynic_parser::executable::{ids::SelectionId, FragmentSpread, Selection};
use indexmap::{IndexMap, IndexSet};

/// A visitor that tracks used fragments in a query, and which selections are
/// ancestors of spreads of those fragments
pub struct FragmentTracker {
    selection_stack: Vec<SelectionId>,
    pub used_fragments: IndexMap<String, IndexSet<SelectionId>>,
}

impl FragmentTracker {
    pub fn new() -> Self {
        FragmentTracker {
            selection_stack: vec![],
            used_fragments: IndexMap::new(),
        }
    }
}

impl super::super::visitor::Visitor for FragmentTracker {
    fn enter_selection(&mut self, id: SelectionId, _selection: Selection<'_>) {
        self.selection_stack.push(id)
    }

    fn exit_selection(&mut self, _id: SelectionId, _selection: Selection<'_>) {
        self.selection_stack.pop();
    }

    fn fragment_spread(&mut self, spread: FragmentSpread<'_>) {
        self.used_fragments
            .entry(spread.fragment_name().to_string())
            .or_default()
            .extend(self.selection_stack.iter().copied())
    }
}
