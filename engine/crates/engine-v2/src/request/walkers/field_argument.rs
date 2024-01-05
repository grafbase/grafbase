use std::ops::Deref;

use engine_value::ConstValue;
use schema::InputValueId;

use crate::request::BoundFieldArgument;

use super::{ExecutorWalkContext, OperationWalker};

pub type BoundFieldArgumentWalker<'a, CtxOrUnit = ()> =
    OperationWalker<'a, &'a BoundFieldArgument, InputValueId, CtxOrUnit>;

impl<'a, C> BoundFieldArgumentWalker<'a, C> {
    // Value in the query, before variable resolution.
    pub fn query_value(&self) -> &engine_value::Value {
        &self.item.value
    }
}

impl<'a> BoundFieldArgumentWalker<'a, ExecutorWalkContext<'a>> {
    pub fn resolved_value(&self) -> ConstValue {
        // not really efficient, but works.
        self.item
            .value
            .clone()
            .into_const_with::<()>(|name| {
                Ok(self
                    .ctx
                    .variables
                    .get(&name)
                    .expect("Would have failed at validation")
                    .value
                    .clone()
                    .unwrap_or_default())
            })
            .unwrap()
    }
}

impl<'a, E> Deref for BoundFieldArgumentWalker<'a, E> {
    type Target = schema::InputValueWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

impl<'a, E> std::fmt::Debug for BoundFieldArgumentWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldArgumentWalker")
            .field("name", &self.name())
            .field("query_value", &self.query_value())
            .finish()
    }
}
