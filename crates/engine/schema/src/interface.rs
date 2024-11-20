use crate::{FieldDefinition, InterfaceDefinition, SubgraphId};

impl<'a> InterfaceDefinition<'a> {
    pub fn is_fully_implemented_in(&self, subgraph_id: SubgraphId) -> bool {
        !self.not_fully_implemented_in_ids.contains(&subgraph_id)
    }

    pub fn is_not_fully_implemented_in(&self, subgraph_id: SubgraphId) -> bool {
        self.not_fully_implemented_in_ids.contains(&subgraph_id)
    }

    pub fn find_field_by_name(&self, name: &str) -> Option<FieldDefinition<'a>> {
        self.fields().find(|field| field.name() == name)
    }

    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_interface_definitions[self.id]
    }
}

impl std::fmt::Debug for InterfaceDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InterfaceDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("fields", &self.fields().map(|field| field.name()).collect::<Vec<_>>())
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
