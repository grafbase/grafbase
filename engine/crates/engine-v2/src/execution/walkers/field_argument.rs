use std::ops::Deref;

use engine_value::ConstValue;
use schema::InputValueId;

use crate::request::BoundFieldArgument;

pub struct FieldArgumentWalker<'a> {
    pub(super) ctx: super::WalkerContext<'a, InputValueId>,
    pub(super) argument: &'a BoundFieldArgument,
}

impl<'a> FieldArgumentWalker<'a> {
    // Value in the query, before variable resolution.
    pub fn query_value(&self) -> &engine_value::Value {
        &self.argument.value
    }

    pub fn resolved_value(&self) -> ConstValue {
        // not really efficient, but works.
        self.argument
            .value
            .clone()
            .into_const_with::<()>(|name| Ok(self.ctx.variables.unchecked_get(&name).value.clone()))
            .unwrap()
    }
}

impl<'a> Deref for FieldArgumentWalker<'a> {
    type Target = schema::InputValueWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ctx.schema_walker
    }
}

impl<'a> std::fmt::Debug for FieldArgumentWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldArgumentWalker")
            .field("name", &self.name())
            .field("query_value", &self.query_value())
            .finish()
    }
}
