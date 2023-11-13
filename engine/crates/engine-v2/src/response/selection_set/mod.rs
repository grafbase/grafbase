use super::ResponseFieldId;

mod read;
mod write;

pub use read::{ReadSelection, ReadSelectionSet};
pub use write::{WriteSelection, WriteSelectionSet};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SelectionSet {
    // sorted by field
    items: Vec<Selection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub field: ResponseFieldId,
    pub subselection: SelectionSet,
}

impl Extend<Selection> for SelectionSet {
    fn extend<T: IntoIterator<Item = Selection>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

impl FromIterator<Selection> for SelectionSet {
    fn from_iter<T: IntoIterator<Item = Selection>>(iter: T) -> Self {
        let mut items = iter.into_iter().collect::<Vec<_>>();
        items.sort_unstable_by_key(|selection| selection.field);
        Self { items }
    }
}

impl IntoIterator for SelectionSet {
    type Item = Selection;

    type IntoIter = <Vec<Selection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a SelectionSet {
    type Item = &'a Selection;

    type IntoIter = <&'a Vec<Selection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl From<Selection> for SelectionSet {
    fn from(selection: Selection) -> Self {
        Self { items: vec![selection] }
    }
}

impl SelectionSet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
