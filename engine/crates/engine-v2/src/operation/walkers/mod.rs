mod argument;
mod field;
mod fragment;
mod inline_fragment;
mod query_path;
mod selection_set;
mod variable;

pub use argument::*;
use engine_parser::types::OperationType;
pub use field::*;
pub use fragment::*;
pub use inline_fragment::*;
use schema::{ObjectWalker, SchemaWalker};
pub use selection_set::*;
#[allow(unused_imports)]
pub use variable::*;

use super::{Operation, Variables};

#[derive(Clone, Copy)]
pub(crate) struct OperationWalker<'a, Item = (), SchemaItem = ()> {
    pub(super) schema_walker: SchemaWalker<'a, SchemaItem>,
    pub(super) operation: &'a Operation,
    pub(super) variables: &'a Variables,
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

    pub(crate) fn is_query(&self) -> bool {
        matches!(self.as_ref().ty, OperationType::Query)
    }

    pub(crate) fn is_mutation(&self) -> bool {
        matches!(self.as_ref().ty, OperationType::Mutation)
    }

    pub(crate) fn selection_set(&self) -> SelectionSetWalker<'a> {
        self.walk(self.operation.root_selection_set_id)
    }

    pub(crate) fn root_object(&self) -> ObjectWalker<'a> {
        self.schema_walker.walk(self.as_ref().root_object_id)
    }
}

impl<'a, I, SI> OperationWalker<'a, I, SI> {
    pub(crate) fn walk<I2>(&self, item: I2) -> OperationWalker<'a, I2, SI>
    where
        SI: Copy,
    {
        OperationWalker {
            schema_walker: self.schema_walker,
            operation: self.operation,
            variables: self.variables,
            item,
        }
    }

    pub fn walk_with<I2, SI2>(&self, item: I2, schema_item: SI2) -> OperationWalker<'a, I2, SI2> {
        OperationWalker {
            schema_walker: self.schema_walker.walk(schema_item),
            operation: self.operation,
            variables: self.variables,
            item,
        }
    }
}
