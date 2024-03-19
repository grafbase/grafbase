mod field;
mod fragment;
mod inline_fragment;
mod query_path;
mod selection_set;

pub use field::*;
pub use fragment::*;
pub use inline_fragment::*;
use schema::SchemaWalker;
pub use selection_set::*;

use super::Operation;

#[derive(Clone, Copy)]
pub(crate) struct OperationWalker<'a, Item = (), SchemaItem = ()> {
    pub(super) operation: &'a Operation,
    pub(super) schema_walker: SchemaWalker<'a, SchemaItem>,
    pub(super) item: Item,
}

impl<'a> std::fmt::Debug for OperationWalker<'a, (), ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationWalker").finish_non_exhaustive()
    }
}

impl<'a, I: Copy, SI> OperationWalker<'a, I, SI>
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

impl<'a> OperationWalker<'a, (), ()> {
    pub(crate) fn as_ref(&self) -> &'a Operation {
        self.operation
    }
}

impl<'a, I, SI> OperationWalker<'a, I, SI> {
    pub(crate) fn walk<I2>(&self, item: I2) -> OperationWalker<'a, I2, SI>
    where
        SI: Copy,
    {
        OperationWalker {
            operation: self.operation,
            schema_walker: self.schema_walker,
            item,
        }
    }

    pub fn walk_with<I2, SI2>(&self, item: I2, schema_item: SI2) -> OperationWalker<'a, I2, SI2> {
        OperationWalker {
            operation: self.operation,
            schema_walker: self.schema_walker.walk(schema_item),
            item,
        }
    }
}
