use super::{NodeSelection, NodeSelectionSet};
use crate::response_graph::{FieldEdgeId, FieldName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputNodeSelectionSet {
    // sorted by field name
    items: Vec<InputNodeSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputNodeSelection {
    pub field: FieldEdgeId,
    pub name: FieldName,
    pub input_name: FieldName,
    pub subselection: InputNodeSelectionSet,
}

impl InputNodeSelectionSet {
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

    pub fn field(&self, name: FieldName) -> Option<&InputNodeSelection> {
        self.items
            .binary_search_by_key(&name, |selection| selection.name)
            .ok()
            .map(|idx| &self.items[idx])
    }
}

impl FromIterator<InputNodeSelection> for InputNodeSelectionSet {
    fn from_iter<T: IntoIterator<Item = InputNodeSelection>>(iter: T) -> Self {
        let mut items = iter.into_iter().collect::<Vec<_>>();
        items.sort_unstable_by_key(|selection| selection.name);
        Self { items }
    }
}

impl From<&InputNodeSelection> for NodeSelection {
    fn from(selection: &InputNodeSelection) -> Self {
        NodeSelection {
            field: selection.field,
            subselection: (&selection.subselection).into(),
        }
    }
}

impl From<&InputNodeSelectionSet> for NodeSelectionSet {
    fn from(selection_set: &InputNodeSelectionSet) -> Self {
        selection_set.items.iter().map(Into::into).collect()
    }
}
