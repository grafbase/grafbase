use crate::ScalarDefinition;

impl ScalarDefinition<'_> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_scalar_definitions[self.id]
    }
}
