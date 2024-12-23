use super::{FieldWalker, OperationWalker};
use crate::operation::BoundSelectionSetId;

pub type SelectionSetWalker<'a> = OperationWalker<'a, BoundSelectionSetId>;

impl<'a> SelectionSetWalker<'a> {
    pub fn fields(self) -> impl Iterator<Item = FieldWalker<'a>> + 'a {
        self.fields_ordered_by_parent_entity_id()
    }

    pub fn fields_ordered_by_parent_entity_id(self) -> impl Iterator<Item = FieldWalker<'a>> + 'a {
        let walker = self.walk(());
        self.as_ref().field_ids.iter().map(move |id| walker.walk(*id))
    }
}

impl std::fmt::Debug for SelectionSetWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionSet")
            .field("fields", &self.fields().collect::<Vec<_>>())
            .finish()
    }
}
