use crate::execution::StrId;

/// Selection set used to read data from the response.
/// Used for plan inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadSelectionSet {
    items: Vec<ReadSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadSelection {
    pub response_key: StrId,
    pub subselection: ReadSelectionSet,
}

impl ReadSelectionSet {
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ReadSelection> {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a ReadSelectionSet {
    type Item = &'a ReadSelection;

    type IntoIter = <&'a Vec<ReadSelection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl FromIterator<ReadSelection> for ReadSelectionSet {
    fn from_iter<T: IntoIterator<Item = ReadSelection>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect::<Vec<_>>(),
        }
    }
}
