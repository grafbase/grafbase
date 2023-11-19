use schema::FieldWalker;

use super::{BoundSelectionSetWalker, OperationFieldArgumentWalker};
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
    pub fn new(
        schema_field: FieldWalker<'a>,
        operation: &'a Operation,
        bound_field: &'a BoundField,
        id: BoundFieldId,
    ) -> Self {
        Self {
            schema_field,
            operation,
            bound_field,
            id,
        }
    }

    pub fn bound_field_id(&self) -> BoundFieldId {
        self.id
    }

    pub fn response_name(&self) -> StrId {
        self.operation[self.bound_field.definition_id].name
    }

    pub fn bound_arguments(&self) -> impl ExactSizeIterator<Item = OperationFieldArgumentWalker<'a>> + 'a {
        let walker = self.schema_field.walk(());
        self.operation[self.bound_field.definition_id]
            .arguments
            .iter()
            .map(move |argument| OperationFieldArgumentWalker {
                argument,
                input_value: walker.walk(argument.input_value_id),
            })
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
            .field("arguments", &self.bound_arguments().collect::<Vec<_>>())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
