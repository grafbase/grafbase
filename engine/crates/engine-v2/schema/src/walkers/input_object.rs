use super::SchemaWalker;
use crate::{InputObjectId, InputValueId, InputValueWalker};

pub type InputObjectWalker<'a> = SchemaWalker<'a, InputObjectId>;

impl<'a> InputObjectWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.input_object(self.schema, self.item)
    }

    pub fn input_fields(&self) -> impl ExactSizeIterator<Item = InputValueWalker<'a>> + 'a {
        let walker = *self;
        let (start, len) = self.schema[self.item].input_fields;
        let start_idx = usize::from(start);
        (start_idx..(start_idx + len))
            .map(InputValueId::from)
            .map(move |id| walker.walk(id))
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
