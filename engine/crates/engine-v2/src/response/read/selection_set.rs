use schema::FieldId;

use crate::response::BoundResponseKey;

/// Selection set used to read data from the response.
/// Used for plan inputs.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ReadSelectionSet {
    items: Vec<ReadField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadField {
    pub bound_response_key: BoundResponseKey,
    pub field_id: FieldId,
    pub subselection: ReadSelectionSet,
}

impl ReadSelectionSet {
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ReadField> {
        self.items.iter()
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
