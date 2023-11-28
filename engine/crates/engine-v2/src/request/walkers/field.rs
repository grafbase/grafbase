use schema::FieldWalker;

use super::BoundSelectionSetWalker;
use crate::{
    execution::StrId,
    request::{BoundField, BoundFieldId, Operation},
};

pub struct BoundFieldWalker<'a> {
    pub(in crate::request) schema_field: FieldWalker<'a>,
    pub(in crate::request) operation: &'a Operation,
    pub(in crate::request) bound_field: &'a BoundField,
    pub(in crate::request) id: BoundFieldId,
}

impl<'a> BoundFieldWalker<'a> {
    pub fn bound_field_id(&self) -> BoundFieldId {
        self.id
    }

    pub fn response_key(&self) -> StrId {
        self.operation[self.bound_field.definition_id].name
    }

    pub fn selection_set(&self) -> BoundSelectionSetWalker<'a> {
        BoundSelectionSetWalker {
            schema: self.schema_field.walk(()),
            operation: self.operation,
            id: self.bound_field.selection_set_id,
        }
    }
}

impl<'a> std::ops::Deref for BoundFieldWalker<'a> {
    type Target = FieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_field
    }
}

impl<'a> std::fmt::Debug for BoundFieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundFieldWalker")
            .field("name", &self.name())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
