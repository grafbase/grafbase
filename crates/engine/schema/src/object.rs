use crate::{
    DeprecatedDirective, FieldDefinition, InterfaceDefinitionId, ObjectDefinition, ObjectDefinitionRecord, SubgraphId,
};

impl ObjectDefinitionRecord {
    pub fn implements_interface_in_subgraph(
        &self,
        subgraph_id: &SubgraphId,
        interface_id: &InterfaceDefinitionId,
    ) -> bool {
        self.join_implement_records
            .binary_search_by(|record| {
                record
                    .subgraph_id
                    .cmp(subgraph_id)
                    .then_with(|| record.interface_id.cmp(interface_id))
            })
            .is_ok()
    }

    pub fn exists_in_subgraph(&self, subgraph_id: &SubgraphId) -> bool {
        self.exists_in_subgraph_ids.contains(subgraph_id)
    }
}

impl<'a> ObjectDefinition<'a> {
    pub fn find_field_by_name(&self, name: &str) -> Option<FieldDefinition<'a>> {
        self.fields().find(|field| field.name() == name)
    }

    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_object_definitions[self.id]
    }

    pub fn has_deprecated(&self) -> Option<DeprecatedDirective<'a>> {
        self.directives().find_map(|directive| directive.as_deprecated())
    }
}

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
            .field("exists_in_subgraph", &self.exists_in_subgraphs().collect::<Vec<_>>())
            .finish()
    }
}
