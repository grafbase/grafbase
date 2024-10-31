use id_newtypes::IdRange;
use schema::Schema;

use crate::{
    operation::{
        ExtraField, Field, FieldArgument, FieldArgumentId, FieldId, Operation, OperationWalker, QueryInputValue,
        SelectionSetId,
    },
    response::ResponseEdge,
};

pub(super) struct BoundOperationAdapter<'a> {
    pub schema: &'a Schema,
    pub operation: &'a mut Operation,
}

impl<'a> query_planning::Operation for BoundOperationAdapter<'a> {
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

    fn create_extra_field(
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
}

impl<'a> BoundOperationAdapter<'a> {
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
