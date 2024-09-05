use crate::UnionDefinition;

impl std::fmt::Debug for UnionDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnionDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field(
                "possible_types",
                &self.possible_types().map(|ty| ty.name()).collect::<Vec<_>>(),
            )
            .field("directives", &self.directives())
            .finish()
    }
}
