use walker::Iter;

use super::{PartitionDataField, PartitionSelectionSet, PartitionTypenameField};

impl<'a> PartitionSelectionSet<'a> {
    pub(crate) fn data_fields(&self) -> impl Iter<Item = PartitionDataField<'a>> + 'a {
        self.data_fields_ordered_by_type_conditions_then_key()
    }

    pub(crate) fn typename_fields(&self) -> impl Iter<Item = PartitionTypenameField<'a>> + 'a {
        self.typename_fields_ordered_by_type_conditions_then_key()
    }
}
