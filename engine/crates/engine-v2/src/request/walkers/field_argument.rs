use std::ops::Deref;

use schema::InputValueId;

use crate::request::BoundFieldArgument;

use super::OperationWalker;

pub type BoundFieldArgumentWalker<'a> = OperationWalker<'a, &'a BoundFieldArgument, InputValueId>;

impl<'a> BoundFieldArgumentWalker<'a> {
    // Value in the query, before variable resolution.
    pub fn query_value(&self) -> &engine_value::Value {
        &self.item.value
    }
}

impl<'a> Deref for BoundFieldArgumentWalker<'a> {
    type Target = schema::InputValueWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

impl<'a> std::fmt::Debug for BoundFieldArgumentWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldArgumentWalker")
            .field("name", &self.name())
            .field("query_value", &self.query_value())
            .finish()
    }
}
