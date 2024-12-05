use crate::{ObjectDefinitionId, SubgraphId, UnionDefinition};

impl UnionDefinition<'_> {
    pub fn is_fully_implemented_in(&self, subgraph_id: SubgraphId) -> bool {
        !self.not_fully_implemented_in_ids.contains(&subgraph_id)
    }

    pub fn has_member_in_subgraph(&self, subgraph_id: SubgraphId, object_id: ObjectDefinitionId) -> bool {
        self.join_member_records
            .binary_search_by(|probe| {
                probe
                    .subgraph_id
                    .cmp(&subgraph_id)
                    .then(probe.member_id.cmp(&object_id))
            })
            .is_ok()
    }

    pub fn is_inaccessible(&self) -> bool {
        self.schema.graph.inaccessible_union_definitions[self.id]
    }

    pub fn has_inaccessible_member(&self) -> bool {
        self.schema.graph.union_has_inaccessible_member[self.id]
    }
}

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
