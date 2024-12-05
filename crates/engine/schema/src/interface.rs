use crate::{FieldDefinition, InterfaceDefinition, ObjectDefinitionId, SubgraphId};

impl<'a> InterfaceDefinition<'a> {
    pub fn has_implementor(&self, id: ObjectDefinitionId) -> bool {
        self.possible_type_ids.binary_search(&id).is_ok()
    }

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

    pub fn has_inaccessible_implementors(&self) -> bool {
        self.schema.graph.interface_has_inaccessible_implementors[self.id]
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
