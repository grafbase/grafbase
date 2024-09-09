use crate::operation::{PreparedOperation, Variables};

mod argument;
mod variable;

pub(crate) use argument::*;
use schema::Schema;

#[derive(Clone, Copy)]
pub(crate) struct PreparedOperationWalker<'a, Item = ()> {
    pub(in crate::operation) schema: &'a Schema,
    pub(in crate::operation) operation: &'a PreparedOperation,
    pub(in crate::operation) variables: &'a Variables,
    pub(in crate::operation) item: Item,
}

impl<'a, I: Copy> PreparedOperationWalker<'a, I>
where
    PreparedOperation: std::ops::Index<I>,
{
    pub fn as_ref(&self) -> &'a <PreparedOperation as std::ops::Index<I>>::Output {
        &self.operation[self.item]
    }
}

impl<'a, I> PreparedOperationWalker<'a, I> {
    pub fn walk<I2>(&self, item: I2) -> PreparedOperationWalker<'a, I2> {
        PreparedOperationWalker {
            schema: self.schema,
            operation: self.operation,
            variables: self.variables,
            item,
        }
    }
}
