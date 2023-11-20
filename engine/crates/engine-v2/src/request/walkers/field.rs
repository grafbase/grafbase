use schema::{FieldId, FieldWalker};

use super::{OperationFieldArgumentWalker, OperationSelectionSetWalker};
use crate::{
    execution::StrId,
    request::{Operation, OperationFieldId, OperationSelectionSet},
};

pub struct OperationFieldWalker<'a> {
    pub(super) operation: &'a Operation,
    pub(super) id: OperationFieldId,
    pub(super) schema_field: FieldWalker<'a>,
    pub(super) selection_set: &'a OperationSelectionSet,
}

impl<'a> OperationFieldWalker<'a> {
    pub fn id(&self) -> FieldId {
        self.schema_field.id
    }

    pub fn name(&self) -> &str {
        self.schema_field.name()
    }

    pub fn response_position(&self) -> usize {
        self.operation[self.id].position
    }

    pub fn response_name(&self) -> StrId {
        self.operation[self.id].name
    }

    pub fn arguments(&self) -> impl ExactSizeIterator<Item = OperationFieldArgumentWalker<'a>> + 'a {
        let field = &self.operation[self.id];
        let walker = self.schema_field.walk(());
        field
            .arguments
            .iter()
            .map(move |argument| OperationFieldArgumentWalker {
                argument,
                input_value: walker.walk(argument.input_value_id),
            })
    }

    pub fn subselection(&self) -> OperationSelectionSetWalker<'a> {
        OperationSelectionSetWalker {
            schema: self.schema_field.walk(()),
            operation: self.operation,
            selection_set: self.selection_set,
        }
    }
}

impl<'a> std::fmt::Debug for OperationFieldWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<OperationFieldWalker<'_>>())
            .field("name", &self.name())
            .field("arguments", &self.arguments().collect::<Vec<_>>())
            .field("subselection", &self.subselection())
            .finish()
    }
}
