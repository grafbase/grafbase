use itertools::Itertools;

use crate::InputObjectDefinition;

impl InputObjectDefinition<'_> {
    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible.input_object_definitions[self.id]
    }
}

impl std::fmt::Debug for InputObjectDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputObjectDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field(
                "input_fields",
                &self
                    .input_fields()
                    .format_with(", ", |field, f| f(&format_args!("{}: {}", field.name(), field.ty()))),
            )
            .field("directives", &self.directives())
            .field("exists_in_subgraphs", &self.exists_in_subgraphs())
            .finish()
    }
}
