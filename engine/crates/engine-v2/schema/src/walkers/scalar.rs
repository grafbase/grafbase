use super::SchemaWalker;
use crate::{ScalarDefinitionId, TypeSystemDirectivesWalker};

pub type ScalarDefinition<'a> = SchemaWalker<'a, ScalarDefinitionId>;

impl<'a> ScalarDefinition<'a> {
    pub fn name(&self) -> &'a str {
        &self.schema[self.as_ref().name_id]
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directive_ids)
    }
}

impl<'a> std::fmt::Debug for ScalarDefinition<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scalar")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .finish()
    }
}
