use engine_value::ConstValue;
use schema::InputValueWalker;

use crate::request::OperationFieldArgument;

pub struct OperationFieldArgumentWalker<'a> {
    pub(super) input_value: InputValueWalker<'a>,
    pub(super) argument: &'a OperationFieldArgument,
}

impl<'a> OperationFieldArgumentWalker<'a> {
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
        f.debug_struct(std::any::type_name::<OperationFieldArgumentWalker<'_>>())
            .field("name", &self.name())
            .field("query_value", &self.query_value())
            .finish()
    }
}
