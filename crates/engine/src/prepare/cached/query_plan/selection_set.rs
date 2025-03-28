use walker::Iter;

use super::{PartitionDataField, PartitionSelectionSet};

impl<'a> PartitionSelectionSet<'a> {
    pub(crate) fn data_fields(&self) -> impl Iter<Item = PartitionDataField<'a>> + 'a {
        self.data_fields_ordered_by_parent_entity_then_key()
    }
}
