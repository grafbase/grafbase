use std::collections::BTreeMap;

use crate::{
    RequiredField, RequiredFieldArguments, RequiredFieldSet, RequiredFieldSetArgumentsId, RequiredFieldSetId, Schema,
};

use super::{
    coerce::{InputValueCoercer, InputValueError},
    ids::IdMaps,
    BuildError, SchemaLocation,
};

#[derive(Default)]
pub(super) struct RequiredFieldSetBuffer(Vec<(SchemaLocation, federated_graph::FieldSet)>);

impl RequiredFieldSetBuffer {
    pub(super) fn push(
        &mut self,
        location: SchemaLocation,
        field_set: federated_graph::FieldSet,
    ) -> RequiredFieldSetId {
        let id = RequiredFieldSetId::from(self.0.len());
        self.0.push((location, field_set));
        id
    }

    pub(super) fn try_insert_into(self, schema: &mut Schema, idmaps: &IdMaps) -> Result<(), BuildError> {
        let mut input_values = std::mem::take(&mut schema.input_values);
        let mut converter = Converter {
            schema,
            idmaps,
            coercer: InputValueCoercer::new(schema, &mut input_values),
            arguments: BTreeMap::new(),
            next_id: 0,
        };

        let mut required_field_sets = Vec::with_capacity(self.0.len());
        for (location, field_set) in self.0 {
            let set =
                converter
                    .convert_set(field_set)
                    .map_err(|err| BuildError::RequiredFieldArgumentCoercionError {
                        location: schema.walk(location).to_string(),
                        err,
                    })?;
            required_field_sets.push(set);
        }

        let mut arguments = converter.arguments.into_iter().collect::<Vec<_>>();
        arguments.sort_unstable_by_key(|(_, id)| *id);
        schema.required_fields_arguments = arguments.into_iter().map(|(args, _)| args).collect();
        schema.required_field_sets = required_field_sets;
        schema.input_values = input_values;
        Ok(())
    }
}

struct Converter<'a> {
    schema: &'a Schema,
    idmaps: &'a IdMaps,
    coercer: InputValueCoercer<'a>,
    arguments: BTreeMap<RequiredFieldArguments, RequiredFieldSetArgumentsId>,
    next_id: u32,
}

impl<'a> Converter<'a> {
    fn convert_set(&mut self, field_set: federated_graph::FieldSet) -> Result<RequiredFieldSet, InputValueError> {
        field_set
            .into_iter()
            .filter_map(|item| self.convert_item(item).transpose())
            .collect::<Result<_, _>>()
    }

    fn convert_item(&mut self, item: federated_graph::FieldSetItem) -> Result<Option<RequiredField>, InputValueError> {
        let Some(field_id) = self.idmaps.field.get(item.field) else {
            return Ok(None);
        };

        let arguments_id = if item.arguments.is_empty() {
            None
        } else {
            let mut arguments = Vec::with_capacity(item.arguments.len());
            for (id, value) in item.arguments {
                let Some(input_value_definition_id) = self.idmaps.input_value.get(id) else {
                    continue;
                };
                let ty = self.schema[input_value_definition_id].ty;
                let input_value_id = self.coercer.coerce(ty, value)?;
                arguments.push((input_value_definition_id, input_value_id));
            }

            let arguments = RequiredFieldArguments(arguments);

            let n = self.arguments.len();
            // Deduplicating arguments allows us to cheaply merge field sets at runtime
            let arguments_id = *self
                .arguments
                .entry(arguments)
                .or_insert_with(|| RequiredFieldSetArgumentsId::from(n));
            Some(arguments_id)
        };

        Ok(Some(RequiredField {
            id: {
                let id = self.next_id;
                self.next_id += 1;
                id.into()
            },
            definition_id: field_id,
            arguments_id,
            subselection: self.convert_set(item.subselection)?,
        }))
    }
}
