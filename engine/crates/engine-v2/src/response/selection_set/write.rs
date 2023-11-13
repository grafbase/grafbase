use super::{Selection, SelectionSet};
use crate::response::ResponseFieldId;

/// Selection set used to write data into the response.
/// Used for plan outputs.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct WriteSelectionSet {
    items: Vec<WriteSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteSelection {
    pub field: ResponseFieldId,
    pub subselection: WriteSelectionSet,
}

impl Extend<WriteSelection> for WriteSelectionSet {
    fn extend<T: IntoIterator<Item = WriteSelection>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

impl FromIterator<WriteSelection> for WriteSelectionSet {
    fn from_iter<T: IntoIterator<Item = WriteSelection>>(iter: T) -> Self {
        let items = iter.into_iter().collect::<Vec<_>>();
        Self { items }
    }
}

impl WriteSelectionSet {
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

    pub fn iter(&self) -> impl Iterator<Item = &WriteSelection> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut WriteSelection> {
        self.items.iter_mut()
    }

    pub fn insert(&mut self, selection: WriteSelection) -> &mut WriteSelection {
        self.items.push(selection);
        let n = self.items.len() - 1;
        &mut self.items[n]
    }
}

impl From<&WriteSelectionSet> for SelectionSet {
    fn from(selection_set: &WriteSelectionSet) -> Self {
        selection_set.items.iter().map(Into::into).collect()
    }
}

impl From<&WriteSelection> for Selection {
    fn from(selection: &WriteSelection) -> Self {
        Selection {
            field: selection.field,
            subselection: (&selection.subselection).into(),
        }
    }
}
