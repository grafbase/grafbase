use super::PartitionSelectionSet;

impl std::fmt::Debug for PartitionSelectionSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keys = &self.ctx.cached.operation.response_keys;
        f.debug_struct("PartitionSelectionSet")
            .field(
                "data_fields",
                &self
                    .data_fields_ordered_by_parent_entity_then_key()
                    .map(|field| &keys[field.response_key])
                    .collect::<Vec<_>>(),
            )
            .field(
                "typename_fields",
                &self
                    .typename_fields()
                    .map(|field| &keys[field.response_key])
                    .collect::<Vec<_>>(),
            )
            .field(
                "lookup_fields",
                &self
                    .lookup_fields()
                    .map(|field| &keys[field.subgraph_key])
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
