mod field;
mod plan;
mod prepared;
mod query_path;
mod selection_set;
mod solved;

use engine_parser::types::OperationType;
pub(crate) use field::*;
pub(crate) use plan::*;
pub(crate) use prepared::*;
use schema::Schema;
pub(crate) use selection_set::*;

use super::Operation;

#[derive(Clone, Copy)]
pub(crate) struct OperationWalker<'a, Item = ()> {
    pub(super) schema: &'a Schema,
    pub(super) operation: &'a Operation,
    pub(super) item: Item,
}

impl<'a> std::fmt::Debug for OperationWalker<'a, ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationWalker").finish_non_exhaustive()
    }
}

impl<'a, I: Copy> OperationWalker<'a, I>
where
    Operation: std::ops::Index<I>,
{
    pub(crate) fn as_ref(&self) -> &'a <Operation as std::ops::Index<I>>::Output {
        &self.operation[self.item]
    }

    pub(crate) fn id(&self) -> I {
        self.item
    }
}

impl<'a> OperationWalker<'a, ()> {
    pub(crate) fn as_ref(&self) -> &'a Operation {
        self.operation
    }

    pub(crate) fn is_query(&self) -> bool {
        matches!(self.as_ref().ty, OperationType::Query)
    }

    pub(crate) fn selection_set(&self) -> SelectionSetWalker<'a> {
        self.walk(self.operation.root_selection_set_id)
    }
}

impl<'a, I> OperationWalker<'a, I> {
    pub(crate) fn walk<I2>(&self, item: I2) -> OperationWalker<'a, I2> {
        OperationWalker {
            schema: self.schema,
            operation: self.operation,
            item,
        }
    }
}
