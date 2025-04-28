use walker::Iter;

use super::{DataField, PartitionSelectionSet};

impl<'a> PartitionSelectionSet<'a> {
    pub(crate) fn data_fields(&self) -> impl Iter<Item = DataField<'a>> + 'a {
        self.data_fields_ordered_by_parent_entity_then_key()
    }
}
