use id_newtypes::IdRange;
use schema::Schema;

use crate::{
    operation::{
        ExtraField, Field, FieldArgument, FieldArgumentId, FieldId, Operation, OperationWalker, QueryInputValue,
        SelectionSetId,
    },
    response::{ResponseEdge, ResponseKey, UnpackedResponseEdge},
};

pub(super) struct OperationAdapter<'a> {
    pub schema: &'a Schema,
    pub operation: &'a mut Operation,
    tmp_response_keys: Vec<ResponseKey>,
}

impl<'a> OperationAdapter<'a> {
    pub fn new(schema: &'a Schema, operation: &'a mut Operation) -> OperationAdapter<'a> {
        OperationAdapter {
            schema,
            operation,
            tmp_response_keys: Vec::new(),
        }
    }
}

impl<'a> query_planning::Operation for OperationAdapter<'a> {
    type FieldId = FieldId;

    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + 'static {
        (0..self.operation.fields.len()).map(FieldId::from)
    }

    fn field_defintion(&self, field_id: Self::FieldId) -> Option<schema::FieldDefinitionId> {
        self.operation[field_id].definition_id()
    }

    fn field_satisfies(&self, field_id: Self::FieldId, requirement: schema::RequiredField<'_>) -> bool {
        let field = OperationWalker {
            schema: self.schema,
            operation: self.operation,
            item: field_id,
        }
        .walk(field_id);
        field.eq(&requirement)
    }

    fn create_potential_extra_field(
        &mut self,
        petitioner_field_id: Self::FieldId,
        requirement: schema::RequiredField<'_>,
    ) -> Self::FieldId {
        let field = ExtraField {
            definition_id: requirement.definition_id,
            argument_ids: self.create_arguments_for(requirement),
            petitioner_location: self.operation[petitioner_field_id].location(),
            // Will be set after planning.
            edge: ResponseEdge::from(0),
            // Not relevant anymore
            selection_set_id: None,
            parent_selection_set_id: SelectionSetId::from(0u16),
        };
        self.operation.fields.push(Field::Extra(field));
        (self.operation.fields.len() - 1).into()
    }

    fn root_selection_set(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + '_ {
        self.operation[self.operation.root_selection_set_id]
            .field_ids_ordered_by_parent_entity_id_then_position
            .iter()
            .copied()
    }

    fn subselection(&self, field_id: Self::FieldId) -> impl ExactSizeIterator<Item = Self::FieldId> + '_ {
        match self.operation[field_id].selection_set_id() {
            Some(id) => self.operation[id]
                .field_ids_ordered_by_parent_entity_id_then_position
                .iter()
                .copied(),
            None => [].iter().copied(),
        }
    }

    fn field_label(&self, field_id: Self::FieldId) -> std::borrow::Cow<'_, str> {
        self.operation.response_keys[self.operation[field_id].response_key()].into()
    }

    fn finalize_selection_set_extra_fields(
        &mut self,
        extra_fields: &[Self::FieldId],
        existing_fields: &[Self::FieldId],
    ) {
        self.tmp_response_keys.clear();
        self.tmp_response_keys.extend(
            existing_fields
                .iter()
                .filter_map(|id| self.operation[*id].response_edge().as_response_key()),
        );
        for extra_field_id in extra_fields {
            let Some(definition_id) = self.operation[*extra_field_id].definition_id() else {
                continue;
            };
            let name = self.schema.walk(definition_id).name();

            let key = 'key: {
                // Key doesn't exist in the operation at all
                let Some(key) = self.operation.response_keys.get(name).map(ResponseKey::from) else {
                    break 'key ResponseKey::from(self.operation.response_keys.get_or_intern(name));
                };

                // Key doesn't exist in the current selection set
                if !self.tmp_response_keys.contains(&key) {
                    break 'key key;
                }

                // Generate a likely unique key
                let hex = hex::encode_upper(u32::from(definition_id).to_be_bytes());
                let short_id = hex.trim_start_matches('0');

                let name = format!("_{}{}", name, short_id);

                let mut i: u8 = 0;
                loop {
                    let candidate = format!("{name}{}", hex::encode_upper(i.to_be_bytes()));
                    if !self.operation.response_keys.contains(&candidate) {
                        break 'key self.operation.response_keys.get_or_intern(&candidate).into();
                    }
                    i += 1;
                }
            };
            if let Field::Extra(field) = &mut self.operation[*extra_field_id] {
                field.edge = UnpackedResponseEdge::ExtraFieldResponseKey(key).pack();
                self.tmp_response_keys.push(key);
            };
        }
    }
}

impl<'a> OperationAdapter<'a> {
    fn create_arguments_for(&mut self, requirement: schema::RequiredField<'_>) -> IdRange<FieldArgumentId> {
        let start = self.operation.field_arguments.len();

        for argument in &requirement.argument_records {
            let input_value_id = self
                .operation
                .query_input_values
                .push_value(QueryInputValue::DefaultValue(argument.value_id));

            self.operation.field_arguments.push(FieldArgument {
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
