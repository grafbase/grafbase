use std::ops::Deref;

use engine_value::ConstValue;
use schema::{InputValueId, SchemaWalker};

use crate::{execution::Variables, request::BoundFieldArgument};

pub struct FieldArgumentWalker<'a> {
    pub(super) input_value: SchemaWalker<'a, InputValueId>,
    pub(super) variables: &'a Variables<'a>,
    pub(super) argument: &'a BoundFieldArgument,
}

impl<'a> FieldArgumentWalker<'a> {
    pub fn new(
        input_value: SchemaWalker<'a, InputValueId>,
        variables: &'a Variables<'a>,
        argument: &'a BoundFieldArgument,
    ) -> Self {
        Self {
            input_value,
            variables,
            argument,
        }
    }

    // Value in the query, before variable resolution.
    pub fn query_value(&self) -> &engine_value::Value {
        &self.argument.value
    }

    pub fn resolved_value(&self) -> ConstValue {
        // not really efficient, but works.
        self.argument
            .value
            .clone()
            .into_const_with::<()>(|name| Ok(self.variables.unchecked_get(&name).value.clone()))
            .unwrap()
    }
}

impl<'a> Deref for FieldArgumentWalker<'a> {
    type Target = schema::InputValueWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.input_value
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
