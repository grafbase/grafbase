use schema::SchemaWalker;

use super::{OperationFieldWalker, OperationWalker};
use crate::request::{Operation, OperationSelectionSet};

pub struct OperationSelectionSetWalker<'a> {
    pub(super) schema: SchemaWalker<'a, ()>,
    pub(super) operation: &'a Operation,
    pub(super) selection_set: &'a OperationSelectionSet,
}

impl<'a> OperationSelectionSetWalker<'a> {
    pub fn is_empty(&self) -> bool {
        self.selection_set.is_empty()
    }

    // Flatten all fields irrelevant of fragments
    pub fn all_fields(&self) -> impl Iterator<Item = OperationFieldWalker<'a>> + 'a {
        let operation = self.operation;
        let walker = self.schema;
        self.selection_set.iter().map(move |selection| {
            let field = walker.walk(operation[selection.operation_field_id].field_id);
            OperationFieldWalker {
                operation,
                id: selection.operation_field_id,
                schema_field: field,
                selection_set: &selection.subselection,
            }
        })
    }

    pub fn new_walker(&self) -> OperationWalker<'a> {
        OperationWalker {
            schema: self.schema,
            operation: self.operation,
        }
    }
}

impl<'a> std::fmt::Debug for OperationSelectionSetWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<OperationSelectionSetWalker<'_>>())
            .field("fields", &self.all_fields().collect::<Vec<_>>())
            .finish()
    }
}
