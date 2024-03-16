use schema::InputTypeWalker;

use crate::operation::VariableDefinitionId;

use super::{PlanInputValue, PlanWalker};

pub type PlanVariable<'a> = PlanWalker<'a, VariableDefinitionId, ()>;

impl<'a> PlanVariable<'a> {
    pub fn name(&self) -> &'a str {
        &self.as_ref().name
    }

    pub fn ty(&self) -> InputTypeWalker<'a> {
        self.schema_walker.walk(self.as_ref().r#type)
    }

    // Before binding the variables this will be the default value or undefined.
    pub fn value(&self) -> PlanInputValue<'a> {
        self.walk(self.as_ref().future_input_value_id)
    }
}
