use crate::{InterfaceDefinition, SubgraphId};

impl<'a> InterfaceDefinition<'a> {
    pub fn is_fully_implemented_in(&self, subgraph_id: SubgraphId) -> bool {
        !self.not_fully_implemented_in_ids.contains(&subgraph_id)
    }

    pub fn is_not_fully_implemented_in(&self, subgraph_id: SubgraphId) -> bool {
        self.not_fully_implemented_in_ids.contains(&subgraph_id)
    }
}

impl std::fmt::Debug for InterfaceDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InterfaceDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("fields", &self.fields().map(|field| field.name()))
            .field(
                "interfaces",
                &self.interfaces().map(|interface| interface.name()).collect::<Vec<_>>(),
            )
            .field(
                "possible_types",
                &self.possible_types().map(|ty| ty.name()).collect::<Vec<_>>(),
            )
            .field("directives", &self.directives())
            .finish()
    }
}
