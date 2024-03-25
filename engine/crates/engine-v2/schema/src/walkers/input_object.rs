use super::SchemaWalker;
use crate::{InputObjectDefinitionId, InputValueDefinitionWalker};

pub type InputObjectWalker<'a> = SchemaWalker<'a, InputObjectDefinitionId>;

impl<'a> InputObjectWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.input_object(self.schema, self.item)
    }

    pub fn input_fields(self) -> impl ExactSizeIterator<Item = InputValueDefinitionWalker<'a>> + 'a {
        self.schema[self.item].input_field_ids.map(move |id| self.walk(id))
    }
}

impl<'a> std::fmt::Debug for InputObjectWalker<'a> {
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
