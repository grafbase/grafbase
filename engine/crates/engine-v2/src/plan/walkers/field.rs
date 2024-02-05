use schema::{FieldId, FieldWalker};

use crate::{
    request::{BoundField, BoundFieldId},
    response::{ResponseEdge, ResponseKey},
};

use super::{PlanInputValue, PlanSelectionSet, PlanWalker};

pub type PlanField<'a> = PlanWalker<'a, BoundFieldId, FieldId>;

impl<'a> PlanField<'a> {
    pub fn selection_set(&self) -> Option<PlanSelectionSet<'a>> {
        self.as_ref()
            .selection_set_id()
            .map(|id| PlanSelectionSet::SelectionSet(self.walk_with(id, ())))
    }

    pub fn response_edge(&self) -> ResponseEdge {
        self.as_ref().response_edge()
    }

    pub fn response_key(&self) -> ResponseKey {
        self.as_ref().response_key()
    }

    pub fn response_key_str(&self) -> &'a str {
        &self.operation_plan.response_keys[self.as_ref().response_key()]
    }

    pub fn arguments(&self) -> impl ExactSizeIterator<Item = PlanInputValue<'a>> + 'a {
        let walker = *self;
        let arguments = match self.as_ref() {
            BoundField::Field { arguments_id, .. } => &self.operation_plan[*arguments_id],
            _ => self.operation_plan.empty_arguments(),
        };
        arguments
            .iter()
            .map(move |argument| walker.walk_with(argument, argument.input_value_id))
    }
}

impl<'a> std::ops::Deref for PlanField<'a> {
    type Target = FieldWalker<'a>;

    fn deref(&self) -> &Self::Target {
        &self.schema_walker
    }
}

impl<'a> std::fmt::Debug for PlanField<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = f.debug_struct("PlanField");
        let name = self.name();
        let response_key = self.response_key_str();
        if response_key != name {
            fmt.field("key", &response_key);
        }
        fmt.field("name", &name)
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
