use std::borrow::Cow;

use id_newtypes::IdRange;
use schema::{ObjectDefinitionId, Schema, SchemaField};
use walker::Walk;

use crate::{
    operation::{
        BoundExtraField, BoundField, BoundFieldArgument, BoundFieldArgumentId, BoundFieldId, BoundOperation,
        QueryInputValueRecord, QueryPosition,
    },
    response::SafeResponseKey,
};

pub(super) struct OperationAdapter<'a> {
    pub schema: &'a Schema,
    pub operation: &'a mut BoundOperation,
    tmp_response_keys: Vec<SafeResponseKey>,
}

impl<'a> OperationAdapter<'a> {
    pub fn new(schema: &'a Schema, operation: &'a mut BoundOperation) -> OperationAdapter<'a> {
        OperationAdapter {
            schema,
            operation,
            tmp_response_keys: Vec::new(),
        }
    }
}

impl<'a> query_solver::Operation for OperationAdapter<'a> {
    type FieldId = BoundFieldId;

    fn field_ids(&self) -> impl ExactSizeIterator<Item = Self::FieldId> + 'static {
        (0..self.operation.fields.len()).map(BoundFieldId::from)
    }

    fn field_definition(&self, field_id: BoundFieldId) -> Option<schema::FieldDefinitionId> {
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
            BoundField::TypeName(field) => Cow::Borrowed(&self.operation.response_keys[field.key.response_key]),
            BoundField::Query(field) => Cow::Borrowed(&self.operation.response_keys[field.key.response_key]),
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

    fn finalize_selection_set_extra_fields(&mut self, extra_fields: &[BoundFieldId], existing_fields: &[BoundFieldId]) {
        self.tmp_response_keys.clear();
        self.tmp_response_keys.extend(
            existing_fields
                .iter()
                .filter_map(|id| self.operation[*id].response_key()),
        );
        for extra_field_id in extra_fields {
            let Some(definition_id) = self.operation[*extra_field_id].definition_id() else {
                continue;
            };
            let name = self.schema.walk(definition_id).name();

            let key = 'key: {
                // Key doesn't exist in the operation at all
                let Some(key) = self.operation.response_keys.get(name) else {
                    break 'key self.operation.response_keys.get_or_intern(name);
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
                        break 'key self.operation.response_keys.get_or_intern(&candidate);
                    }
                    i += 1;
                }
            };
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
            BoundField::TypeName(field) => usize::from(field.key.query_position.unwrap()),
            BoundField::Query(field) => usize::from(field.key.query_position.unwrap()),
            BoundField::Extra(_) => QueryPosition::EXTRA,
        }
    }
}

impl<'a> OperationAdapter<'a> {
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
