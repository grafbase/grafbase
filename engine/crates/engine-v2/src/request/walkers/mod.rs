use schema::SchemaWalker;

use super::{Operation, OperationSelectionSet};

mod argument;
mod field;
mod selection_set;

pub use argument::*;
pub use field::*;
pub use selection_set::*;

#[derive(Clone, Copy)]
pub struct OperationWalker<'a> {
    pub schema: SchemaWalker<'a, ()>,
    pub operation: &'a Operation,
}

impl<'ctx> OperationWalker<'ctx> {
    pub fn walk<'a>(&self, selection_set: &'a OperationSelectionSet) -> OperationSelectionSetWalker<'a>
    where
        'ctx: 'a,
    {
        OperationSelectionSetWalker {
            schema: self.schema,
            operation: self.operation,
            selection_set,
        }
    }
}
