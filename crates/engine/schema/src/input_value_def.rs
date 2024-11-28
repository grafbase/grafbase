use crate::InputValueDefinition;

impl InputValueDefinition<'_> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_input_value_definitions[self.id]
    }
}
