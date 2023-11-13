use crate::response::ResponseStringId;

/// Selection set used to read data from the response.
/// Used for plan inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadSelectionSet {
    // sorted by name
    items: Vec<ReadSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadSelection {
    pub name: ResponseStringId,
    pub subselection: ReadSelectionSet,
}

impl ReadSelectionSet {
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

    pub fn find_field(&self, name: ResponseStringId) -> Option<&ReadSelection> {
        self.items
            .binary_search_by_key(&name, |selection| selection.name)
            .ok()
            .map(|idx| &self.items[idx])
    }
}

impl FromIterator<ReadSelection> for ReadSelectionSet {
    fn from_iter<T: IntoIterator<Item = ReadSelection>>(iter: T) -> Self {
        let mut items = iter.into_iter().collect::<Vec<_>>();
        items.sort_unstable_by_key(|selection| selection.name);
        Self { items }
    }
}
