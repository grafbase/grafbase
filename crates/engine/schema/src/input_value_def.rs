use crate::InputValueDefinition;

impl InputValueDefinition<'_> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_input_value_definitions[self.id]
    }
}

impl std::fmt::Display for InputValueDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name(), self.ty())?;
        if let Some(default_value) = self.default_value() {
            write!(f, " = {}", default_value)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for InputValueDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputValueDefinition")
            .field("name", &self.name())
            .field("ty", &self.ty())
            .field("default_value", &self.default_value())
            .field("directives", &self.directives())
            .finish()
    }
}
