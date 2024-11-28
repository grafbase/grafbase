use std::collections::BTreeMap;

use id_newtypes::IdRange;

use crate::{FieldSetId, Graph, SchemaFieldId};

use super::{
    coerce::{InputValueCoercer, InputValueError},
    BuildContext, BuildError, FieldSetItemRecord, FieldSetRecord, InputValueDefinitionId, SchemaFieldArgumentRecord,
    SchemaFieldRecord, SchemaLocation,
};

#[derive(Default)]
pub(super) struct FieldSetsBuilder(Vec<(SchemaLocation, federated_graph::SelectionSet)>);

impl FieldSetsBuilder {
    pub(super) fn push(&mut self, location: SchemaLocation, field_set: federated_graph::SelectionSet) -> FieldSetId {
        let id = FieldSetId::from(self.0.len());
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
            field_arguments: Vec::new(),
        };

        let mut field_sets = Vec::with_capacity(self.0.len());
        for (location, field_set) in self.0 {
            let set =
                converter
                    .convert_set(field_set)
                    .map_err(|err| BuildError::RequiredFieldArgumentCoercionError {
                        location: location.to_string(ctx),
                        err,
                    })?;
            field_sets.push(set);
        }

        let field_arguments = converter.field_arguments;
        let mut fields = converter.deduplicated_fields.into_iter().collect::<Vec<_>>();
        fields.sort_unstable_by_key(|(_, id)| *id);
        graph.fields = fields.into_iter().map(|(field, _)| field).collect();
        graph.field_arguments = field_arguments;
        graph.field_sets = field_sets;
        graph.input_values = input_values;
        Ok(())
    }
}

struct Converter<'a> {
    ctx: &'a BuildContext,
    graph: &'a Graph,
    coercer: InputValueCoercer<'a>,
    deduplicated_fields: BTreeMap<SchemaFieldRecord, SchemaFieldId>,
    field_arguments: Vec<SchemaFieldArgumentRecord>,
}

impl Converter<'_> {
    fn convert_set(&mut self, field_set: federated_graph::SelectionSet) -> Result<FieldSetRecord, InputValueError> {
        let mut out = Vec::with_capacity(field_set.len());
        self.convert_set_rec(field_set, &mut out)?;
        Ok(out.into_iter().collect())
    }

    fn convert_set_rec(
        &mut self,
        field_set: federated_graph::SelectionSet,
        out: &mut Vec<FieldSetItemRecord>,
    ) -> Result<(), InputValueError> {
        let mut stack = vec![field_set];
        while let Some(field_set) = stack.pop() {
            for item in field_set.0 {
                match item {
                    federated_graph::Selection::Field(field) => {
                        out.extend(self.convert_item(field.field_id, field.arguments, field.subselection)?)
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
        arguments: Vec<(federated_graph::InputValueDefinitionId, federated_graph::Value)>,
        subselection: federated_graph::SelectionSet,
    ) -> Result<Option<FieldSetItemRecord>, InputValueError> {
        let field_definition_id = field_id.into();

        let mut federated_arguments = arguments
            .into_iter()
            .map(|(id, value)| (InputValueDefinitionId::from(id), value))
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
                let ty = self.graph[definition_id].ty_record;
                let value_id = self.coercer.coerce(ty, value)?;
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
            alias_id: self.graph[field_definition_id].name_id,
            id,
            subselection_record: self.convert_set(subselection)?,
        }))
    }
}
