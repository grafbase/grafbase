use super::SchemaWalker;
use crate::{InputObjectDefinitionId, InputValueDefinition, TypeSystemDirectivesWalker};

pub type InputObjectDefinition<'a> = SchemaWalker<'a, InputObjectDefinitionId>;

impl<'a> InputObjectDefinition<'a> {
    pub fn name(&self) -> &'a str {
        &self.schema[self.as_ref().name_id]
    }

    pub fn input_fields(self) -> impl ExactSizeIterator<Item = InputValueDefinition<'a>> + 'a {
        self.schema[self.item]
            .input_field_ids
            .into_iter()
            .map(move |id| self.walk(id))
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directive_ids)
    }
}

impl<'a> std::fmt::Debug for InputObjectDefinition<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputObject")
            .field("id", &usize::from(self.item))
            .field("name", &self.name())
            .field(
                "input_fields",
                &self
                    .input_fields()
                    .map(|f| (f.name(), f.ty().to_string()))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
