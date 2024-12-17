use std::collections::HashMap;

use operation::ResponseKey;
use petgraph::visit::EdgeRef;
use schema::{CompositeTypeId, DefinitionId, FieldDefinitionId, StringId, SubgraphId};
use walker::Walk;

use crate::{
    query::{FieldFlags, SolutionEdge, SolutionNode},
    QueryFieldId,
};

use super::SolvedQueryWithoutPostProcessing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FieldRenameConsistencyKey {
    FieldWithDistinctType {
        key: ResponseKey,
        field_definition_id: FieldDefinitionId,
    },
    FieldNamedTypename {
        output_definition_id: DefinitionId,
    },
}

impl SolvedQueryWithoutPostProcessing<'_> {
    pub(super) fn adjust_response_keys_to_avoid_collisions(&mut self) {
        let mut field_renames = HashMap::<FieldRenameConsistencyKey, ResponseKey>::new();
        let mut existing_fields = Vec::new();
        let mut extra_fields = Vec::new();
        let mut stack = vec![(
            CompositeTypeId::from(self.operation.root_object_id),
            SubgraphId::Introspection,
            self.root_node_ix,
        )];
        while let Some((parent_type, subgraph_id, node)) = stack.pop() {
            extra_fields.clear();
            existing_fields.clear();
            for edge in self.graph.edges(node) {
                if !matches!(edge.weight(), SolutionEdge::Field | SolutionEdge::QueryPartition) {
                    continue;
                }
                match self.graph[edge.target()] {
                    SolutionNode::Field { id, flags, .. } => {
                        if flags.contains(FieldFlags::EXTRA) {
                            extra_fields.push((subgraph_id, id));
                        } else {
                            existing_fields.push((subgraph_id, id));
                        }
                        if let Some(parent_type) = self[id]
                            .definition_id
                            .and_then(|id| id.walk(self.schema).ty().definition_id.as_composite_type())
                        {
                            stack.push((parent_type, subgraph_id, edge.target()));
                        }
                    }
                    SolutionNode::QueryPartition {
                        resolver_definition_id, ..
                    } => {
                        let subgraph_id = resolver_definition_id.walk(self.schema).subgraph_id();
                        for second_degree_edge in self.graph.edges(edge.target()) {
                            if !matches!(
                                second_degree_edge.weight(),
                                SolutionEdge::Field | SolutionEdge::QueryPartition
                            ) {
                                continue;
                            }
                            if let SolutionNode::Field { id, flags, .. } = self.graph[second_degree_edge.target()] {
                                if flags.contains(FieldFlags::EXTRA) {
                                    extra_fields.push((subgraph_id, id));
                                } else {
                                    existing_fields.push((subgraph_id, id));
                                }
                                if let Some(parent_type) = self[id]
                                    .definition_id
                                    .and_then(|id| id.walk(self.schema).ty().definition_id.as_composite_type())
                                {
                                    stack.push((parent_type, subgraph_id, second_degree_edge.target()));
                                }
                            }
                        }
                    }
                    SolutionNode::Root => (),
                }
            }
            self.adjust_response_keys_to_avoid_collision_in_selection_set(
                parent_type,
                &extra_fields,
                &existing_fields,
                &mut field_renames,
            );
        }
    }

    fn adjust_response_keys_to_avoid_collision_in_selection_set(
        &mut self,
        parent_type: CompositeTypeId,
        extra_fields: &[(SubgraphId, QueryFieldId)],
        existing_fields: &[(SubgraphId, QueryFieldId)],
        field_renames: &mut HashMap<FieldRenameConsistencyKey, ResponseKey>,
    ) {
        let mut selection_set_keys = Vec::with_capacity(extra_fields.len() + existing_fields.len());
        selection_set_keys.extend(existing_fields.iter().filter_map(|(_, id)| self[*id].key));

        // If the parent type is an object we don't need to deal with distinct types as we'll only
        // query a single object from the subgraph.
        if !parent_type.is_object() {
            for (subgraph_id, query_field_id) in existing_fields.iter().copied() {
                let query_field = &self[query_field_id];
                let Some((key, definition_id)) = query_field.key.zip(query_field.definition_id) else {
                    continue;
                };
                let definition = definition_id.walk(self.schema);

                let new_key = if definition
                    .subgraph_type_records
                    .iter()
                    .any(|record| record.subgraph_id == subgraph_id)
                {
                    self.generate_new_key(
                        Some(FieldRenameConsistencyKey::FieldWithDistinctType {
                            key,
                            field_definition_id: definition.id,
                        }),
                        &selection_set_keys,
                        definition.name_id,
                        field_renames,
                    )
                } else if &self.operation.response_keys[key] == "__typename" {
                    self.generate_new_key(
                        Some(FieldRenameConsistencyKey::FieldNamedTypename {
                            output_definition_id: definition.ty().definition_id,
                        }),
                        &selection_set_keys,
                        definition.name_id,
                        field_renames,
                    )
                } else {
                    continue;
                };

                self[query_field_id].subgraph_key = Some(new_key);
                selection_set_keys.push(new_key);
            }
        }

        for (_, id) in extra_fields {
            let Some(definition_id) = self[*id].definition_id else {
                continue;
            };
            let key = self.generate_new_key(
                None,
                &selection_set_keys,
                definition_id.walk(self.schema).name_id,
                field_renames,
            );

            self[*id].key = Some(key);
            selection_set_keys.push(key);
        }
    }

    /// There are three cases today for renaming a field:
    ///  1. The field has a distinct type in the subgraph than the one we have in the supergraph.
    ///  2. Field is named `__typename` which will clash with the `__typename` field we expect to
    ///     retrieve the type name.
    ///  3. We're adding a extra field to satisfy a requirement.
    ///
    ///
    /// 1. Distinct type
    /// ----------------
    /// Generate a new response key for a field with an distinct type, but compatible, than the super-graph.
    /// This can happen when a subgraph A defines:
    ///
    /// ```ignore,graphql
    /// type User {
    ///   id: ID @shareable
    /// }
    /// ```
    ///
    /// and subgraph B defines:
    ///
    /// ```ignore,graphql
    /// type User @key(fields: "id") {
    ///   id: ID!
    /// }
    /// type Admin {
    ///   id: ID
    /// }
    /// ```
    ///
    /// In this case the super-graph will use `id: ID`. A problem arises if we query something
    /// like this on an `UserOrAdmin` union:
    /// ```ignore,graphql
    /// {
    ///   userOrAdmin {
    ///     ... on User {
    ///       id
    ///       name
    ///     }
    ///     ... on Admin {
    ///       id
    ///       name
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// In this case the subgraph will complain that `User.id` and `Admin.id` have different types.
    /// So the federated SDL will actually track this with:
    ///
    /// ```ignore,graphql
    ///   id: ID @join__field(graph: A, type: "ID") @join__field(graph: B, type: "ID!")
    /// ```
    ///
    /// Whenever a user request a field with a distinct type such as `User.id` we need to rename
    /// it in the subgraph query. As we need to deal with selection set merging we have to be
    /// consistent with this name, so we generate an unique name per `(SafeResponseKey, FieldDefinitionId)`
    /// as we may have aliases with different arguments.
    ///
    /// Luckily for us we won't need to deal with interface fields when response merging. An
    /// interface needs to be consistent with its object so if `User` implemented an interface with
    /// an `id` field it would either be inconsistent at the super-graph or subgraph level. So we
    /// will never need to worry about merging with interface fields.
    ///
    /// 2. Typename
    /// -----------
    /// Similar to the previous case we need to ensure consistent renaming across selection sets to
    /// ensure proper merging. However, we do need to handle interfaces this time. So we the output
    /// DefinitionId as the key to ensure that we end up merging fields even if field with
    /// different definitions end up being merged together.
    ///
    /// 3. Extra field
    /// --------------
    /// Contrary to the other cases those fields will never be exposed to the client so their
    /// response key doesn't matter, but it needs to be unique within the selection
    /// set we send to the subgraph to generate a proper query string.
    ///
    /// We don't need to keep track of this later, because we identify requirements by
    /// their associated `SchemaFieldId` rather than
    fn generate_new_key(
        &mut self,
        rename_consistency_key: Option<FieldRenameConsistencyKey>,
        selection_set_keys: &[ResponseKey],
        name_suggestion: StringId,
        field_renames: &mut HashMap<FieldRenameConsistencyKey, ResponseKey>,
    ) -> ResponseKey {
        if let Some(key) = rename_consistency_key.as_ref().and_then(|k| field_renames.get(k)) {
            return *key;
        }

        let name = name_suggestion.walk(self.schema);

        // Key doesn't exist in the operation at all
        let Some(key) = self.operation.response_keys.get(name) else {
            let key = self.operation.response_keys.get_or_intern(name);
            if let Some(field_rename_key) = rename_consistency_key {
                field_renames.insert(field_rename_key, key);
            }
            return key;
        };

        // if we don't need to care about being consistent with the renaming across selection set,
        // we can just return the key if it's not present within the current one.
        // This is only present to generate nicer subgraph queries.
        if rename_consistency_key.is_none() && !selection_set_keys.contains(&key) {
            return key;
        }

        // Generate a likely unique key
        let hex = hex::encode_upper(u32::from(name_suggestion).to_be_bytes());
        let short_id = hex.trim_start_matches('0');

        let name = format!("_{}{}", name, short_id);

        let mut i: u8 = 0;
        loop {
            let candidate = format!("{name}{}", hex::encode_upper(i.to_be_bytes()));

            // Key doesn't exist in the operation at all
            if !self.operation.response_keys.contains(&candidate) {
                let key = self.operation.response_keys.get_or_intern(&candidate);
                if let Some(field_rename_key) = rename_consistency_key {
                    field_renames.insert(field_rename_key, key);
                }
                return key;
            };

            i = i.wrapping_add(1);
            if i == 0 {
                unimplemented!("Couldn not find a unique field name.")
            }
        }
    }
}
