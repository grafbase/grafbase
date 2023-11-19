use core::num::NonZeroUsize;

use crate::{execution::StrId, request::BoundFieldDefinitionId};

/// Selection set used to write data into the response.
/// Used for plan outputs.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct WriteSelectionSet {
    dense_capacity: Option<NonZeroUsize>,
    items: Vec<WriteSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteSelection {
    pub operation_field_id: BoundFieldDefinitionId,
    pub source_name: StrId,
    pub response_position: usize,
    pub response_name: StrId,
    pub subselection: WriteSelectionSet,
}

impl WriteSelectionSet {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn new(dense_capacity: Option<NonZeroUsize>, items: Vec<WriteSelection>) -> Self {
        Self { dense_capacity, items }
    }

    pub fn iter(&self) -> impl Iterator<Item = &WriteSelection> {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a WriteSelectionSet {
    type Item = &'a WriteSelection;

    type IntoIter = <&'a Vec<WriteSelection> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}
