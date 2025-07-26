use crate::ScalarDefinition;

impl ScalarDefinition<'_> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible.scalar_definitions[self.id]
    }
}
