use walker::Iter;

use super::{DataField, SelectionSet, TypenameField};

impl<'a> SelectionSet<'a> {
    pub(crate) fn data_fields(&self) -> impl Iter<Item = DataField<'a>> + 'a {
        self.data_fields_ordered_by_parent_entity_id_then_key()
    }

    pub(crate) fn typename_fields(&self) -> impl Iter<Item = TypenameField<'a>> + 'a {
        self.typename_fields_ordered_by_type_condition_id_then_key()
    }
}
