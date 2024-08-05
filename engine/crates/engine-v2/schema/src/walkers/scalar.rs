use super::SchemaWalker;
use crate::{ScalarDefinitionId, TypeSystemDirectivesWalker};

pub type ScalarDefinitionWalker<'a> = SchemaWalker<'a, ScalarDefinitionId>;

impl<'a> ScalarDefinitionWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.scalar(self.schema, self.item)
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directives)
    }
}

impl<'a> std::fmt::Debug for ScalarDefinitionWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scalar")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .finish()
    }
}
