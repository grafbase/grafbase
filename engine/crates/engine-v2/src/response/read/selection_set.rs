use crate::response::ResponseEdge;

/// Selection set used to read data from the response.
/// Used for plan inputs.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ReadSelectionSet {
    items: Vec<ReadField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadField {
    pub edge: ResponseEdge,
    pub name: String,
    pub subselection: ReadSelectionSet,
}

impl ReadSelectionSet {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn extend_disjoint(&mut self, other: Self) {
        self.items.extend(other.items);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<'a> IntoIterator for &'a ReadSelectionSet {
    type Item = &'a ReadField;

    type IntoIter = <&'a Vec<ReadField> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl FromIterator<ReadField> for ReadSelectionSet {
    fn from_iter<T: IntoIterator<Item = ReadField>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect::<Vec<_>>(),
        }
    }
}
