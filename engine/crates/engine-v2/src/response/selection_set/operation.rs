use crate::response::ResponseFieldId;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct OperationSelectionSet {
    // sorted by field
    items: Vec<OperationSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationSelection {
    pub field: ResponseFieldId,
    pub subselection: OperationSelectionSet,
}

impl Extend<OperationSelection> for OperationSelectionSet {
    fn extend<T: IntoIterator<Item = OperationSelection>>(&mut self, iter: T) {
        self.items.extend(iter);
    }
}

impl FromIterator<OperationSelection> for OperationSelectionSet {
    fn from_iter<T: IntoIterator<Item = OperationSelection>>(iter: T) -> Self {
        let mut items = iter.into_iter().collect::<Vec<_>>();
        items.sort_unstable_by_key(|selection| selection.field);
        Self { items }
    }
}

impl IntoIterator for OperationSelectionSet {
    type Item = OperationSelection;

    type IntoIter = <Vec<OperationSelection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a> IntoIterator for &'a OperationSelectionSet {
    type Item = &'a OperationSelection;

    type IntoIter = <&'a Vec<OperationSelection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl From<OperationSelection> for OperationSelectionSet {
    fn from(selection: OperationSelection) -> Self {
        Self { items: vec![selection] }
    }
}

impl OperationSelectionSet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
