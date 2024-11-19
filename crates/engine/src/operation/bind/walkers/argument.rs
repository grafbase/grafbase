use id_newtypes::{IdRange, IdRangeIterator};
use schema::InputValueDefinition;
use walker::Walk;

use crate::operation::{input_value::InputValueContext, BoundFieldArgumentId, QueryInputValue, Variables};

use super::OperationWalker;

pub type FieldArgumentWalker<'a> = OperationWalker<'a, BoundFieldArgumentId>;

impl<'a> FieldArgumentWalker<'a> {
    pub fn value(&self, variables: &'a Variables) -> QueryInputValue<'a> {
        self.operation.query_input_values[self.as_ref().input_value_id].walk(InputValueContext {
            schema: self.schema,
            query_input_values: &self.operation.query_input_values,
            variables,
        })
    }

    pub fn definition(&self) -> InputValueDefinition<'a> {
        self.schema.walk(self.operation[self.item].input_value_definition_id)
    }
}

impl std::fmt::Debug for FieldArgumentWalker<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldArgumentWalker")
            .field("name", &self.definition().name())
            .finish_non_exhaustive()
    }
}

pub type FieldArgumentsWalker<'a> = OperationWalker<'a, IdRange<BoundFieldArgumentId>>;

impl<'a> FieldArgumentsWalker<'a> {
    #[expect(dead_code)] // Will use this later
    pub fn is_empty(&self) -> bool {
        self.item.is_empty()
    }
}

impl<'a> IntoIterator for FieldArgumentsWalker<'a> {
    type Item = FieldArgumentWalker<'a>;

    type IntoIter = FieldArgumentsIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FieldArgumentsIterator(self.walk(self.item.into_iter()))
    }
}

pub(crate) struct FieldArgumentsIterator<'a>(OperationWalker<'a, IdRangeIterator<BoundFieldArgumentId>>);

impl<'a> Iterator for FieldArgumentsIterator<'a> {
    type Item = FieldArgumentWalker<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.item.next().map(|id| self.0.walk(id))
    }
}

impl ExactSizeIterator for FieldArgumentsIterator<'_> {
    fn len(&self) -> usize {
        self.0.item.len()
    }
}
