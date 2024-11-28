use crate::InputObjectDefinition;

impl InputObjectDefinition<'_> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_input_object_definitions[self.id]
    }
}
