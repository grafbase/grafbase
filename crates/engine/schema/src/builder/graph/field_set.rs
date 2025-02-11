use id_newtypes::IdRange;

use crate::SchemaFieldId;

use super::{
    builder::InputValueError, FieldSetItemRecord, FieldSetRecord, GraphContext, SchemaFieldArgumentRecord,
    SchemaFieldRecord,
};

impl GraphContext<'_> {
    pub fn convert_field_set(
        &mut self,
        field_set: &federated_graph::SelectionSet,
    ) -> Result<FieldSetRecord, InputValueError> {
        let mut out = Vec::with_capacity(field_set.len());
        self.convert_set_rec(field_set, &mut out)?;
        Ok(out.into_iter().collect())
    }

    fn convert_set_rec(
        &mut self,
        field_set: &federated_graph::SelectionSet,
        out: &mut Vec<FieldSetItemRecord>,
    ) -> Result<(), InputValueError> {
        let mut stack = vec![field_set];
        while let Some(field_set) = stack.pop() {
            for item in &field_set.0 {
                match item {
                    federated_graph::Selection::Field(field) => {
                        out.extend(self.convert_item(field.field_id, &field.arguments, &field.subselection)?)
                    }
                    federated_graph::Selection::InlineFragment { on: _, subselection } => stack.push(subselection),
                }
            }
        }

        Ok(())
    }

    fn convert_item(
        &mut self,
        field_id: federated_graph::FieldId,
        arguments: &[(federated_graph::InputValueDefinitionId, federated_graph::Value)],
        subselection: &federated_graph::SelectionSet,
    ) -> Result<Option<FieldSetItemRecord>, InputValueError> {
        let field_definition_id = field_id.into();

        let mut federated_arguments = arguments
            .iter()
            .map(|(id, value)| (self.input_value_mapping[id], value))
            .collect::<Vec<_>>();
        let mut field = SchemaFieldRecord {
            definition_id: field_definition_id,
            sorted_argument_ids: IdRange::empty(),
        };

        let start = self.field_arguments.len();
        for definition_id in self.graph[field_definition_id].argument_ids {
            let input_value_definition = &self.graph[definition_id];
            if let Some(index) = federated_arguments.iter().position(|(id, _)| *id == definition_id) {
                let (_, value) = federated_arguments.swap_remove(index);
                let value_id = self.coerce(definition_id, value.clone())?;
                self.field_arguments.push(SchemaFieldArgumentRecord {
                    definition_id,
                    value_id,
                });
            } else if let Some(value_id) = input_value_definition.default_value_id {
                self.field_arguments.push(SchemaFieldArgumentRecord {
                    definition_id,
                    value_id,
                });
            } else if input_value_definition.ty_record.wrapping.is_required() {
                return Err(InputValueError::MissingRequiredArgument(
                    self.ctx.strings[input_value_definition.name_id].clone(),
                ));
            }
        }

        self.field_arguments[start..].sort_unstable_by_key(|arg| arg.definition_id);
        field.sorted_argument_ids = IdRange::from(start..self.field_arguments.len());

        let n = self.deduplicated_fields.len();
        // Deduplicating arguments allows us to cheaply merge field sets at runtime
        let id = *self
            .deduplicated_fields
            .entry(field)
            .or_insert_with(|| SchemaFieldId::from(n));

        Ok(Some(FieldSetItemRecord {
            field_id: id,
            subselection_record: self.convert_field_set(subselection)?,
        }))
    }
}
