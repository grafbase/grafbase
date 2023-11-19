use engine_value::ConstValue;
use schema::InputValueWalker;

use crate::request::BoundFieldArgument;

pub struct OperationFieldArgumentWalker<'a> {
    pub(super) input_value: InputValueWalker<'a>,
    pub(super) argument: &'a BoundFieldArgument,
}

impl<'a> OperationFieldArgumentWalker<'a> {
    pub fn new(input_value: InputValueWalker<'a>, argument: &'a BoundFieldArgument) -> Self {
        Self { input_value, argument }
    }

    pub fn name(&self) -> &str {
        self.input_value.name()
    }

    // Value in the query, before variable resolution.
    pub fn query_value(&self) -> &engine_value::Value {
        &self.argument.value
    }

    pub fn resolved_value(&self) -> ConstValue {
        self.argument
            .value
            .clone()
            .into_const_with(|_name| Err("no variables yet"))
            .unwrap()
    }
}

impl<'a> std::fmt::Debug for OperationFieldArgumentWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationFieldArgumentWalker")
            .field("name", &self.name())
            .field("query_value", &self.query_value())
            .finish()
    }
}
