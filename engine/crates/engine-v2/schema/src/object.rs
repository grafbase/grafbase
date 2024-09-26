use crate::{InterfaceDefinitionId, ObjectDefinition, SubgraphId};

impl<'a> ObjectDefinition<'a> {
    pub fn subgraph_implements_interface(
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

    pub fn is_resolvable_in(&self, subgraph_id: &SubgraphId) -> bool {
        self.only_resolvable_in_ids.is_empty() || self.only_resolvable_in_ids.binary_search(subgraph_id).is_ok()
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
            .finish()
    }
}
