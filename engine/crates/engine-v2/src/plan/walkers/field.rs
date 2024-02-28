use schema::{FieldId, FieldWalker};

use crate::{
    request::BoundFieldId,
    response::{ResponseEdge, ResponseKey},
};

use super::{PlanFieldArgument, PlanInputValue, PlanSelectionSet, PlanWalker};

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
        self.operation_plan
            .response_keys
            .try_resolve(self.response_key())
            .unwrap()
    }

    pub fn arguments(self) -> impl ExactSizeIterator<Item = PlanFieldArgument<'a>> + 'a {
        self.as_ref()
            .argument_ids()
            .map(move |id| self.walk_with(id, self.operation_plan[id].input_value_definition_id))
    }

    pub fn get_arg(&self, name: &str) -> PlanInputValue<'a> {
        self.arguments()
            .find_map(|arg| if arg.name() == name { Some(arg.value()) } else { None })
            .unwrap_or_else(|| self.walk_with(self.operation_plan.input_values.undefined_value_id(), ()))
    }

    #[track_caller]
    pub fn get_arg_as<T: serde::Deserialize<'a>>(&self, name: &str) -> T {
        T::deserialize(self.get_arg(name)).expect("Invalid argument type.")
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
        let arguments = self.arguments().collect::<Vec<_>>();
        if !arguments.is_empty() {
            fmt.field("arguments", &arguments);
        }
        fmt.field("name", &name)
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
