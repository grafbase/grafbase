mod field;
mod field_argument;
mod fragment;
mod inline_fragment;
mod operation_limits;
mod query_path;
mod selection_set;
mod variables;

pub use field::*;
pub use fragment::*;
pub use inline_fragment::*;
use schema::SchemaWalker;
pub use selection_set::*;
pub use variables::*;

use super::{Operation, TypeCondition, VariableDefinitionId};

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

    pub(crate) fn variable_definition(&self, name: &str) -> Option<VariableDefinitionWalker<'a>> {
        for (i, variable) in self.operation.variable_definitions.iter().enumerate() {
            if variable.name == name {
                return Some(self.walk(VariableDefinitionId::from(i)));
            }
        }
        None
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

pub(crate) fn type_condition_name<I>(schema: SchemaWalker<'_, I>, type_condition: TypeCondition) -> &str {
    match type_condition {
        TypeCondition::Interface(interface_id) => schema.walk(interface_id).name(),
        TypeCondition::Object(object_id) => schema.walk(object_id).name(),
        TypeCondition::Union(union_id) => schema.walk(union_id).name(),
    }
}
