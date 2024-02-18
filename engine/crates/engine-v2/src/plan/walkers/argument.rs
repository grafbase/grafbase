use schema::{InputValueDefinitionId, InputValueDefinitionWalker};

use crate::request::BoundFieldArgumentId;

use super::{PlanInputValue, PlanWalker};

pub type PlanFieldArgument<'a> = PlanWalker<'a, BoundFieldArgumentId, InputValueDefinitionId>;

impl<'a> PlanFieldArgument<'a> {
    pub fn value(&self) -> PlanInputValue<'a> {
        self.walk_with(self.as_ref().input_value_id, ())
    }
}

impl<'a> std::ops::Deref for PlanFieldArgument<'a> {
    type Target = InputValueDefinitionWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

impl std::fmt::Debug for PlanFieldArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldArgument")
            .field("name", &self.name())
            .field("value", &self.value())
            .finish()
    }
}
