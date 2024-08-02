use schema::SchemaWalker;

use crate::operation::{PreparedOperation, Variables};

mod argument;
mod variable;

pub(crate) use argument::*;

#[derive(Clone, Copy)]
pub(crate) struct PreparedOperationWalker<'a, Item = (), SchemaItem = ()> {
    pub(in crate::operation) schema_walker: SchemaWalker<'a, SchemaItem>,
    pub(in crate::operation) operation: &'a PreparedOperation,
    pub(in crate::operation) variables: &'a Variables,
    pub(in crate::operation) item: Item,
}

impl<'a, I: Copy, SI> PreparedOperationWalker<'a, I, SI>
where
    PreparedOperation: std::ops::Index<I>,
{
    pub fn as_ref(&self) -> &'a <PreparedOperation as std::ops::Index<I>>::Output {
        &self.operation[self.item]
    }
}

impl<'a, I> PreparedOperationWalker<'a, I, ()> {
    pub fn walk<I2>(&self, item: I2) -> PreparedOperationWalker<'a, I2, ()> {
        PreparedOperationWalker {
            schema_walker: self.schema_walker,
            operation: self.operation,
            variables: self.variables,
            item,
        }
    }
}

impl<'a, I, SI> PreparedOperationWalker<'a, I, SI> {
    pub fn walk_with<I2, SI2>(&self, item: I2, schema_item: SI2) -> PreparedOperationWalker<'a, I2, SI2> {
        PreparedOperationWalker {
            schema_walker: self.schema_walker.walk(schema_item),
            operation: self.operation,
            variables: self.variables,
            item,
        }
    }
}
