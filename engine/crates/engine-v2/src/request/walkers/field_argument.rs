use std::ops::Deref;

use engine_value::ConstValue;
use schema::InputValueId;

use crate::request::BoundFieldArgument;

use super::{HasVariables, OperationWalker};

pub type BoundFieldArgumentWalker<'a, Extension = ()> =
    OperationWalker<'a, &'a BoundFieldArgument, InputValueId, Extension>;

impl<'a, E> BoundFieldArgumentWalker<'a, E> {
    // Value in the query, before variable resolution.
    pub fn query_value(&self) -> &engine_value::Value {
        &self.inner.value
    }

    pub fn resolved_value(&self) -> ConstValue
    where
        E: HasVariables,
    {
        // not really efficient, but works.
        self.inner
            .value
            .clone()
            .into_const_with::<()>(|name| {
                Ok(self
                    .ext
                    .variables()
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
        &self.schema
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
