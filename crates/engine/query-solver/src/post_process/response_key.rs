use std::collections::HashMap;

use operation::{Operation, ResponseKey};
use petgraph::visit::EdgeRef;
use schema::{CompositeTypeId, DefinitionId, FieldDefinitionId, Schema, StringId, SubgraphId};
use walker::Walk;

use crate::{
    query::{Edge, Node},
    solve::CrudeSolvedQuery,
    QueryFieldId,
};

pub(super) fn adjust_response_keys_to_avoid_collisions(
    schema: &Schema,
    operation: &mut Operation,
    query: &mut CrudeSolvedQuery,
) {
    KeyGenerationContext {
        schema,
        operation,
        query,
        field_renames: HashMap::new(),
    }
    .adjust_response_keys_to_avoid_collision()
}

struct KeyGenerationContext<'a> {
    schema: &'a Schema,
    operation: &'a mut Operation,
    query: &'a mut CrudeSolvedQuery,
    field_renames: HashMap<FieldRenameConsistencyKey, ResponseKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FieldRenameConsistencyKey {
    FieldWithDistinctType {
        response_key: ResponseKey,
        field_definition_id: FieldDefinitionId,
    },
    FieldNamedTypename {
        output_definition_id: DefinitionId,
    },
}

#[derive(Default)]
struct SelectionSetContext {
    response_keys: Vec<ResponseKey>,
    fields: Vec<(SubgraphId, QueryFieldId)>,
}

impl SelectionSetContext {
    fn clear(&mut self) {
        self.response_keys.clear();
        self.fields.clear();
    }

    fn push_field(&mut self, subgraph_id: SubgraphId, id: QueryFieldId, key: Option<ResponseKey>) {
        self.fields.push((subgraph_id, id));
        if let Some(key) = key {
            self.response_keys.push(key);
        }
    }
}

impl KeyGenerationContext<'_> {
    fn adjust_response_keys_to_avoid_collision(&mut self) {
        let mut selection_set = SelectionSetContext::default();
        let mut stack = vec![(
            self.query.root_node_ix,
            CompositeTypeId::from(self.operation.root_object_id),
            SubgraphId::Introspection,
        )];
        while let Some((parent_node_ix, output_type_id, subgraph_id)) = stack.pop() {
            selection_set.clear();
            for edge in self.query.graph.edges(parent_node_ix) {
                if !matches!(edge.weight(), Edge::Field | Edge::QueryPartition) {
                    continue;
                }
                match self.query.graph[edge.target()] {
                    Node::Field { id, .. } => {
                        let field = &self.query[id];
                        selection_set.push_field(subgraph_id, id, field.response_key);
                        if let Some(ty) = field.selection_set_id.map(|id| self.query[id].output_type_id) {
                            stack.push((edge.target(), ty, subgraph_id));
                        }
                    }
                    Node::QueryPartition {
                        resolver_definition_id, ..
                    } => {
                        let subgraph_id = resolver_definition_id.walk(self.schema).subgraph_id();
                        for second_degree_edge in self.query.graph.edges(edge.target()) {
                            if !matches!(second_degree_edge.weight(), Edge::Field) {
                                continue;
                            }
                            let field_node_ix = second_degree_edge.target();
                            if let Node::Field { id, .. } = self.query.graph[field_node_ix] {
                                let field = &self.query[id];
                                selection_set.push_field(subgraph_id, id, field.response_key);
                                if let Some(ty) = field.selection_set_id.map(|id| self.query[id].output_type_id) {
                                    stack.push((field_node_ix, ty, subgraph_id));
                                }
                            }
                        }
                    }
                    Node::Root | Node::Typename => (),
                }
            }
            self.adjust_response_keys_to_avoid_collision_in_selection_set(output_type_id, &mut selection_set);
        }
    }

    fn adjust_response_keys_to_avoid_collision_in_selection_set(
        &mut self,
        parent_type: CompositeTypeId,
        selection_set: &mut SelectionSetContext,
    ) {
        // Generating a different subgraph key to prevent collisions.
        for (subgraph_id, query_field_id) in selection_set.fields.iter().copied() {
            let query_field = &self.query[query_field_id];
            let Some(response_key) = query_field.response_key else {
                continue;
            };
            let definition = query_field.definition_id.walk(self.schema);

            // If the parent type is an object we don't need to deal with distinct types as we'll only
            // query a single object from the subgraph.
            let new_response_key = if !parent_type.is_object()
                && definition
                    .subgraph_type_records
                    .iter()
                    .any(|record| record.subgraph_id == subgraph_id)
            {
                self.generate_new_key(
                    selection_set,
                    Some(FieldRenameConsistencyKey::FieldWithDistinctType {
                        response_key,
                        field_definition_id: definition.id,
                    }),
                    definition.name_id,
                )
            } else if &self.operation.response_keys[response_key] == "__typename" {
                self.generate_new_key(
                    selection_set,
                    Some(FieldRenameConsistencyKey::FieldNamedTypename {
                        output_definition_id: definition.ty().definition_id,
                    }),
                    definition.name_id,
                )
            } else {
                continue;
            };

            self.query[query_field_id].subgraph_key = Some(new_response_key);
            selection_set.response_keys.push(new_response_key);
        }

        // Generating a key for extra fields we kept.
        'extra_fields: for (_, id) in &selection_set.fields {
            let query_field = &self.query[*id];
            if query_field.response_key.is_some() {
                continue;
            }
            let definition = query_field.definition_id.walk(self.schema).as_ref();

            // We may request the same field but from different objects (ex: Cat.name and Dog.name), if so we just re-use the
            // existing name for clarity.
            for (_, other_field_id) in &selection_set.fields {
                let other_field = &self.query[*other_field_id];
                let Some(other_key) = other_field.response_key else {
                    continue;
                };
                let other_definition = other_field.definition_id.walk(self.schema).as_ref();

                // if different object fields but implement the same interface fields
                if other_definition.name_id == definition.name_id
                    && other_definition.ty_record == definition.ty_record
                    && query_field.definition_id != other_field.definition_id
                    && definition.parent_entity_id != other_definition.parent_entity_id
                    && definition.parent_entity_id.is_object()
                    && other_definition.parent_entity_id.is_object()
                {
                    self.query[*id].response_key = Some(other_key);
                    continue 'extra_fields;
                }
            }
            let key = self.generate_new_key(selection_set, None, definition.name_id);

            self.query[*id].response_key = Some(key);
            selection_set.response_keys.push(key);
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
        selection_set: &SelectionSetContext,
        rename_consistency_key: Option<FieldRenameConsistencyKey>,
        name_suggestion: StringId,
    ) -> ResponseKey {
        if let Some(key) = rename_consistency_key.as_ref().and_then(|k| self.field_renames.get(k)) {
            return *key;
        }

        let name = name_suggestion.walk(self.schema);

        // Key doesn't exist in the operation at all
        let Some(key) = self.operation.response_keys.get(name) else {
            let key = self.operation.response_keys.get_or_intern(name);
            if let Some(field_rename_key) = rename_consistency_key {
                self.field_renames.insert(field_rename_key, key);
            }
            return key;
        };

        // if we don't need to care about being consistent with the renaming across selection set,
        // we can just return the key if it's not present within the current one.
        // This is only present to generate nicer subgraph queries.
        if rename_consistency_key.is_none() && !selection_set.response_keys.contains(&key) {
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
                    self.field_renames.insert(field_rename_key, key);
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
