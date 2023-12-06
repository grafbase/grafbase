use super::{BoundAnyFieldDefinitionWalker, BoundSelectionSetWalker, OperationWalker, PlanFilter};
use crate::{request::BoundFieldId, response::ResponseKey};

pub type BoundFieldWalker<'a, Extension = ()> = OperationWalker<'a, BoundFieldId, (), Extension>;

impl<'a, E: Copy> BoundFieldWalker<'a, E> {
    pub fn response_key_str(&self) -> &str {
        &self.operation.response_keys[self.bound_response_key]
    }

    pub fn response_key(&self) -> ResponseKey {
        self.bound_response_key.into()
    }

    pub fn definition(&self) -> BoundAnyFieldDefinitionWalker<'a, E> {
        self.walk_with(self.definition_id, ())
    }
}

impl<'a, E: PlanFilter + Copy> BoundFieldWalker<'a, E> {
    pub fn selection_set(&self) -> Option<BoundSelectionSetWalker<'a, E>> {
        self.selection_set_id
            .filter(|id| self.ext.selection_set(*id))
            .map(|id| self.walk(id))
    }
}

impl<'a, E: PlanFilter + Copy> std::fmt::Debug for BoundFieldWalker<'a, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundFieldWalker")
            .field("definition", &self.definition())
            .field("selection_set", &self.selection_set())
            .finish()
    }
}
