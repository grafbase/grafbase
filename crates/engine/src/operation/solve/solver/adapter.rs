use std::borrow::Cow;

use id_newtypes::IdRange;
use im::HashMap;
use schema::{
    CompositeTypeId, FieldDefinition, FieldDefinitionId, ObjectDefinitionId, Schema, SchemaField, SubgraphId,
};
use walker::Walk;

use crate::{
    operation::{
        BoundExtraField, BoundField, BoundFieldArgument, BoundFieldArgumentId, BoundFieldId, BoundOperation,
        QueryInputValueRecord, QueryPosition,
    },
    response::{ResponseKeys, SafeResponseKey},
};

pub(super) struct OperationAdapter<'a> {
    pub schema: &'a Schema,
    pub operation: &'a mut BoundOperation,
    tmp_response_keys: Vec<SafeResponseKey>,
    field_with_distinct_type_to_unique_key: HashMap<FieldDefinitionId, SafeResponseKey>,
}

impl<'a> OperationAdapter<'a> {
    pub fn new(schema: &'a Schema, operation: &'a mut BoundOperation) -> OperationAdapter<'a> {
        OperationAdapter {
            schema,
            operation,
            tmp_response_keys: Vec::new(),
            field_with_distinct_type_to_unique_key: HashMap::new(),
        }
    }
}

impl<'a> query_solver::Operation for OperationAdapter<'a> {
    type FieldId = BoundFieldId;

    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + 'static {
        (0..self.operation.fields.len()).map(BoundFieldId::from)
    }

    fn field_definition(&self, field_id: BoundFieldId) -> Option<FieldDefinitionId> {
        self.operation[field_id].definition_id()
    }

    fn field_is_equivalent_to(&self, field_id: BoundFieldId, requirement: SchemaField<'_>) -> bool {
        self.operation.walker_with(self.schema).walk(field_id).eq(&requirement)
    }

    fn create_potential_extra_field(
        &mut self,
        petitioner_field_id: BoundFieldId,
        requirement: SchemaField<'_>,
    ) -> Self::FieldId {
        let field = BoundExtraField {
            key: None, // added later if used
            definition_id: requirement.definition_id,
            argument_ids: self.create_arguments_for(requirement),
            petitioner_location: self.operation[petitioner_field_id].location(),
        };
        self.operation.fields.push(BoundField::Extra(field));
        (self.operation.fields.len() - 1).into()
    }

    fn root_selection_set(&self) -> impl ExactSizeIterator<Item = BoundFieldId> + '_ {
        self.operation[self.operation.root_selection_set_id]
            .field_ids
            .iter()
            .copied()
    }

    fn subselection(&self, field_id: BoundFieldId) -> impl ExactSizeIterator<Item = Self::FieldId> + '_ {
        match &self.operation[field_id] {
            BoundField::Query(field) => field
                .selection_set_id
                .map(|id| self.operation[id].field_ids.iter().copied())
                .unwrap_or_else(|| [].iter().copied()),
            _ => [].iter().copied(),
        }
    }

    fn field_label(&self, field_id: BoundFieldId) -> std::borrow::Cow<'_, str> {
        match &self.operation[field_id] {
            BoundField::TypeName(field) => Cow::Borrowed(&self.operation.response_keys[field.key]),
            BoundField::Query(field) => Cow::Borrowed(&self.operation.response_keys[field.key]),
            // For extra fields we didn't create a response key yet.
            BoundField::Extra(field) => {
                if let Some(key) = field.key {
                    Cow::Borrowed(&self.operation.response_keys[key])
                } else {
                    field.definition_id.walk(self.schema).name().into()
                }
            }
        }
    }

    fn finalize_selection_set(
        &mut self,
        parent_type: CompositeTypeId,
        extra_fields: &mut [(SubgraphId, Self::FieldId)],
        existing_fields: &mut [(SubgraphId, Self::FieldId)],
    ) {
        self.tmp_response_keys.clear();
        self.tmp_response_keys.extend(
            existing_fields
                .iter()
                .filter_map(|(_, id)| self.operation[*id].response_key()),
        );

        // If the parent type is an object we don't need to deal with distinct types as we'll only
        // query a single object from the subgraph.
        if !parent_type.is_object() {
            for (subgraph_id, field_id) in existing_fields.iter().copied() {
                let Some(definition) = self.operation[field_id].definition_id().walk(self.schema) else {
                    continue;
                };
                if definition.distinct_type_in_ids.contains(&subgraph_id) {
                    let key = self.generate_new_distinct_type_key(definition);
                    if let BoundField::Query(field) = &mut self.operation[field_id] {
                        field.subgraph_key = key;
                        self.tmp_response_keys.push(key);
                    };
                }
            }
        }

        for (_, extra_field_id) in extra_fields {
            let Some(definition_id) = self.operation[*extra_field_id].definition_id() else {
                continue;
            };
            let key = generate_new_extra_field_key(
                &mut self.operation.response_keys,
                &self.tmp_response_keys,
                definition_id.walk(self.schema),
            );
            if let BoundField::Extra(field) = &mut self.operation[*extra_field_id] {
                field.key = Some(key);
                self.tmp_response_keys.push(key);
            };
        }
    }

    fn root_object_id(&self) -> ObjectDefinitionId {
        self.operation.root_object_id
    }

    fn field_query_position(&self, field_id: Self::FieldId) -> usize {
        match &self.operation[field_id] {
            BoundField::TypeName(field) => usize::from(field.query_position),
            BoundField::Query(field) => usize::from(field.query_position),
            BoundField::Extra(_) => QueryPosition::EXTRA,
        }
    }
}

impl<'a> OperationAdapter<'a> {
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
    /// consistent with this name, so we generate an unique name per `FieldDefinitionId`.
    ///
    /// Luckily for us we won't need to deal with interface fields when response merging. An
    /// interface needs to be consistent with its object so if `User` implemented an interface with
    /// an `id` field it would either be inconsistent at the super-graph or subgraph level. So we
    /// will never need to worry about merging with interface fields.
    fn generate_new_distinct_type_key(&mut self, definition: FieldDefinition<'_>) -> SafeResponseKey {
        if let Some(key) = self.field_with_distinct_type_to_unique_key.get(&definition.id) {
            return *key;
        }

        let name = definition.name();

        // Key doesn't exist in the operation at all
        if !self.operation.response_keys.contains(name) {
            let key = self.operation.response_keys.get_or_intern(name);
            self.field_with_distinct_type_to_unique_key.insert(definition.id, key);
            return key;
        }

        // Generate a likely unique key
        let hex = hex::encode_upper(u32::from(definition.id).to_be_bytes());
        let short_id = hex.trim_start_matches('0');

        let name = format!("_{}{}", name, short_id);

        let mut i: u8 = 0;
        loop {
            let candidate = format!("{name}{}", hex::encode_upper(i.to_be_bytes()));

            // Key doesn't exist in the operation at all
            if !self.operation.response_keys.contains(&candidate) {
                let key = self.operation.response_keys.get_or_intern(&candidate);
                self.field_with_distinct_type_to_unique_key.insert(definition.id, key);
                return key;
            };

            i = i.wrapping_add(1);
            if i == 0 {
                unimplemented!("Couldn not find a unique field name.")
            }
        }
    }

    fn create_arguments_for(&mut self, requirement: SchemaField<'_>) -> IdRange<BoundFieldArgumentId> {
        let start = self.operation.field_arguments.len();

        for argument in requirement.sorted_arguments() {
            let input_value_id = self
                .operation
                .query_input_values
                .push_value(QueryInputValueRecord::DefaultValue(argument.value_id));

            self.operation.field_arguments.push(BoundFieldArgument {
                name_location: None,
                value_location: None,
                input_value_id,
                input_value_definition_id: argument.definition_id,
            });
        }

        let end = self.operation.field_arguments.len();
        (start..end).into()
    }
}

/// Generating a response key for an extra field, a field we added during planning to satisfy a
/// requirement. Contrary to `generate_new_distinct_type_key` those fields will never be exposed to
/// the client so their response key doesn't matter, but it needs to be unique within the selection
/// set we send to the subgraph to generate a proper query string.
///
/// We don't need to keep track of this later, because we identify requirements by
/// their associated `SchemaFieldId` rather than
fn generate_new_extra_field_key(
    response_keys: &mut ResponseKeys,
    selection_set: &[SafeResponseKey],
    definition: FieldDefinition<'_>,
) -> SafeResponseKey {
    let name = definition.name();

    // Key doesn't exist in the operation at all
    let Some(key) = response_keys.get(name) else {
        return response_keys.get_or_intern(name);
    };

    // Key doesn't exist in the current selection set
    if !selection_set.contains(&key) {
        return key;
    }

    // Generate a likely unique key
    let hex = hex::encode_upper(u32::from(definition.id).to_be_bytes());
    let short_id = hex.trim_start_matches('0');

    let name = format!("_{}{}", name, short_id);

    let mut i: u8 = 0;
    loop {
        let candidate = format!("{name}{}", hex::encode_upper(i.to_be_bytes()));

        // Key doesn't exist in the operation at all
        let Some(key) = response_keys.get(&candidate) else {
            return response_keys.get_or_intern(&candidate);
        };

        // Key doesn't exist in the current selection set
        if !selection_set.contains(&key) {
            return key;
        }

        i = i.wrapping_add(1);
        if i == 0 {
            unimplemented!("Couldn not find a unique field name.")
        }
    }
}
