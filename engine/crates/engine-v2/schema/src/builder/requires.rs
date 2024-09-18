use std::collections::BTreeMap;

use crate::{
    Graph, RequiredFieldId, RequiredFieldRecord, RequiredFieldSetId, RequiredFieldSetItemRecord, RequiredFieldSetRecord,
};

use super::{
    coerce::{InputValueCoercer, InputValueError},
    BuildContext, BuildError, RequiredFieldArgumentRecord, SchemaLocation,
};

#[derive(Default)]
pub(super) struct RequiredFieldSetBuffer(Vec<(SchemaLocation, federated_graph::SelectionSet)>);

impl RequiredFieldSetBuffer {
    pub(super) fn push(
        &mut self,
        location: SchemaLocation,
        field_set: federated_graph::SelectionSet,
    ) -> RequiredFieldSetId {
        let id = RequiredFieldSetId::from(self.0.len());
        self.0.push((location, field_set));
        id
    }

    pub(super) fn try_insert_into(self, ctx: &BuildContext, graph: &mut Graph) -> Result<(), BuildError> {
        let mut input_values = std::mem::take(&mut graph.input_values);
        let mut converter = Converter {
            ctx,
            graph,
            coercer: InputValueCoercer::new(ctx, graph, &mut input_values),
            deduplicated_fields: BTreeMap::new(),
        };

        let mut required_field_sets = Vec::with_capacity(self.0.len());
        for (location, field_set) in self.0 {
            let set =
                converter
                    .convert_set(field_set)
                    .map_err(|err| BuildError::RequiredFieldArgumentCoercionError {
                        location: location.to_string(ctx),
                        err,
                    })?;
            required_field_sets.push(set);
        }

        let mut arguments = converter.deduplicated_fields.into_iter().collect::<Vec<_>>();
        arguments.sort_unstable_by_key(|(_, id)| *id);
        graph.required_fields = arguments.into_iter().map(|(field, _)| field).collect();
        graph.required_field_sets = required_field_sets;
        graph.input_values = input_values;
        Ok(())
    }
}

struct Converter<'a> {
    ctx: &'a BuildContext,
    graph: &'a Graph,
    coercer: InputValueCoercer<'a>,
    deduplicated_fields: BTreeMap<RequiredFieldRecord, RequiredFieldId>,
}

impl<'a> Converter<'a> {
    fn convert_set(
        &mut self,
        field_set: federated_graph::SelectionSet,
    ) -> Result<RequiredFieldSetRecord, InputValueError> {
        field_set
            .into_iter()
            .filter_map(|item| self.convert_item(item).transpose())
            .collect::<Result<_, _>>()
    }

    fn convert_item(
        &mut self,
        item: federated_graph::Selection,
    ) -> Result<Option<RequiredFieldSetItemRecord>, InputValueError> {
        match item {
            federated_graph::Selection::Field {
                field,
                arguments,
                subselection,
            } => self.convert_field_selection(field, arguments, subselection),
            federated_graph::Selection::InlineFragment { on, subselection } => {
                self.convert_inline_fragment(on, subselection)
            }
        }
    }

    fn convert_field_selection(
        &mut self,
        field: federated_graph::FieldId,
        arguments: Vec<(federated_graph::InputValueDefinitionId, federated_graph::Value)>,
        subselection: federated_graph::SelectionSet,
    ) -> Result<Option<RequiredFieldSetItemRecord>, InputValueError> {
        let Some(definition_id) = self.ctx.idmaps.field.get(field) else {
            return Ok(None);
        };

        let mut federated_arguments = arguments
            .into_iter()
            .filter_map(|(id, value)| {
                let definition_id = self.ctx.idmaps.input_value.get(id)?;
                Some((definition_id, value))
            })
            .collect::<Vec<_>>();
        let mut field = RequiredFieldRecord {
            definition_id,
            argument_records: Vec::with_capacity(federated_arguments.len()),
        };

        for definition_id in self.graph[definition_id].argument_ids {
            let input_value_definition = &self.graph[definition_id];
            if let Some(index) = federated_arguments.iter().position(|(id, _)| *id == definition_id) {
                let (_, value) = federated_arguments.swap_remove(index);
                let ty = self.graph[definition_id].ty_record;
                let value_id = self.coercer.coerce(ty, value)?;
                field.argument_records.push(RequiredFieldArgumentRecord {
                    definition_id,
                    value_id,
                });
            } else if let Some(value_id) = input_value_definition.default_value_id {
                field.argument_records.push(RequiredFieldArgumentRecord {
                    definition_id,
                    value_id,
                });
            } else if input_value_definition.ty_record.wrapping.is_required() {
                return Err(InputValueError::MissingRequiredArgument(
                    self.ctx.strings[input_value_definition.name_id].clone(),
                ));
            }
        }

        let n = self.deduplicated_fields.len();
        // Deduplicating arguments allows us to cheaply merge field sets at runtime
        let id = *self
            .deduplicated_fields
            .entry(field)
            .or_insert_with(|| RequiredFieldId::from(n));

<<<<<<< HEAD
        Ok(Some(RequiredFieldSetItemRecord {
            field_id: id,
            subselection: self.convert_set(subselection)?,
        }))
=======
        Ok(Some(RequiredFieldSetItemRecord::Field(
            super::RequiredFieldSetFieldRecord {
                field_id: id,
                subselection: self.convert_set(subselection)?,
            },
        )))
    }

    fn convert_inline_fragment(
        &mut self,
        on: federated_graph::Definition,
        subselection: Vec<federated_graph::Selection>,
    ) -> Result<Option<RequiredFieldSetItemRecord>, InputValueError> {
        Ok(Some(RequiredFieldSetItemRecord::InlineFragment(
            super::RequiredFieldSetInlineFragmentRecord {
                on: on.into(),
                subselection: self.convert_set(subselection)?,
            },
        )))
>>>>>>> df73acdd7 (wip)
    }
}
