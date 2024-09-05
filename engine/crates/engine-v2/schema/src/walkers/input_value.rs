use super::SchemaWalker;
use crate::{InputValueDefinitionId, Type, TypeSystemDirectivesWalker};

pub type InputValueDefinition<'a> = SchemaWalker<'a, InputValueDefinitionId>;

impl<'a> InputValueDefinition<'a> {
    pub fn name(&self) -> &'a str {
        &self.schema[self.as_ref().name_id]
    }

    pub fn ty(&self) -> Type<'a> {
        self.walk(self.as_ref().ty)
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directive_ids)
    }
}

impl<'a> std::fmt::Debug for InputValueDefinition<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputValue")
            .field("name", &self.name())
            .field("ty", &self.ty())
            .finish()
    }
}
