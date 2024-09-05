use crate::{FieldDefinition, InputValueDefinition};

impl<'a> FieldDefinition<'a> {
    pub fn argument_by_name(&self, name: &str) -> Option<InputValueDefinition<'a>> {
        self.arguments().find(|arg| arg.name() == name)
    }
}
