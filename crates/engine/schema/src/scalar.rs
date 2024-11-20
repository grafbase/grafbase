use crate::ScalarDefinition;

impl<'a> ScalarDefinition<'a> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_scalar_definitions[self.id]
    }
}
