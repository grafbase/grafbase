use crate::InputObjectDefinition;

impl<'a> InputObjectDefinition<'a> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_input_object_definitions[self.id]
    }
}
