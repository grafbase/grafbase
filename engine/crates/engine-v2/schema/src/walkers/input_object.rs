use super::SchemaWalker;
use crate::{InputObjectDefinitionId, InputValueDefinitionWalker, TypeSystemDirectivesWalker};

pub type InputObjectDefinitionWalker<'a> = SchemaWalker<'a, InputObjectDefinitionId>;

impl<'a> InputObjectDefinitionWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.input_object(self.schema, self.item)
    }

    pub fn input_fields(self) -> impl ExactSizeIterator<Item = InputValueDefinitionWalker<'a>> + 'a {
        self.schema[self.item]
            .input_field_ids
            .into_iter()
            .map(move |id| self.walk(id))
    }

    pub fn directives(&self) -> TypeSystemDirectivesWalker<'a> {
        self.walk(self.as_ref().directives)
    }
}

impl<'a> std::fmt::Debug for InputObjectDefinitionWalker<'a> {
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
