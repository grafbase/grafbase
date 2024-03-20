use schema::FieldDefinitionId;

use crate::operation::FieldId;

use super::{PlanSelectionSet, PlanWalker};

pub type PlanField<'a> = PlanWalker<'a, FieldId, FieldDefinitionId>;

impl<'a> PlanField<'a> {
    pub fn selection_set(&self) -> Option<PlanSelectionSet<'a>> {
        self.as_ref()
            .selection_set_id()
            .map(|id| PlanSelectionSet::SelectionSet(self.walk_with(id, ())))
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
        let arguments = self.arguments().collect::<Vec<_>>();
        if !arguments.is_empty() {
            fmt.field("arguments", &arguments);
        }
        fmt.field("name", &name)
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
