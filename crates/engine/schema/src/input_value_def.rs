use crate::InputValueDefinition;

impl<'a> InputValueDefinition<'a> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_input_value_definitions[self.id]
    }
}
