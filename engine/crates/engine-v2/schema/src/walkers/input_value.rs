use super::SchemaWalker;
use crate::{InputValueId, TypeWalker};

pub type InputValueWalker<'a> = SchemaWalker<'a, InputValueId>;

impl<'a> InputValueWalker<'a> {
    pub fn name(&self) -> &'a str {
        self.names.input_value(self.id)
    }

    pub fn ty(&self) -> TypeWalker<'a> {
        self.walk(self.type_id)
    }
}
