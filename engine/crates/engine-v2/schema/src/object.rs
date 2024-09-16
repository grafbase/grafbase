use crate::ObjectDefinition;

impl std::fmt::Debug for ObjectDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field(
                "interfaces",
                &self.interfaces().map(|interface| interface.name()).collect::<Vec<_>>(),
            )
            .field("directives", &self.directives())
            .field("fields", &self.fields().map(|f| f.name()).collect::<Vec<_>>())
            .finish()
    }
}
