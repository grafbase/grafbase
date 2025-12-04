use crate::ScalarDefinition;

impl ScalarDefinition<'_> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible.scalar_definitions[self.id]
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self.name(), "Boolean" | "Float" | "ID" | "Int" | "String")
    }
}
