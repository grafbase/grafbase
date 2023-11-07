use super::{NodeSelection, NodeSelectionSet};
use crate::response_graph::FieldEdgeId;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct OutputNodeSelectionSet {
    items: Vec<OutputNodeSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputNodeSelection {
    pub field: FieldEdgeId,
    pub subselection: OutputNodeSelectionSet,
}

impl Extend<OutputNodeSelection> for OutputNodeSelectionSet {
    fn extend<T: IntoIterator<Item = OutputNodeSelection>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

impl FromIterator<OutputNodeSelection> for OutputNodeSelectionSet {
    fn from_iter<T: IntoIterator<Item = OutputNodeSelection>>(iter: T) -> Self {
        let items = iter.into_iter().collect::<Vec<_>>();
        Self { items }
    }
}

impl OutputNodeSelectionSet {
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

    pub fn iter(&self) -> impl Iterator<Item = &OutputNodeSelection> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut OutputNodeSelection> {
        self.items.iter_mut()
    }

    pub fn insert(&mut self, selection: OutputNodeSelection) -> &mut OutputNodeSelection {
        self.items.push(selection);
        let n = self.items.len() - 1;
        &mut self.items[n]
    }
}

impl From<&OutputNodeSelectionSet> for NodeSelectionSet {
    fn from(selection_set: &OutputNodeSelectionSet) -> Self {
        selection_set.items.iter().map(Into::into).collect()
    }
}

impl From<&OutputNodeSelection> for NodeSelection {
    fn from(selection: &OutputNodeSelection) -> Self {
        NodeSelection {
            field: selection.field,
            subselection: (&selection.subselection).into(),
        }
    }
}
