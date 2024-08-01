use grafbase_telemetry::metrics::OperationMetricsAttributes;
use itertools::Itertools;
use schema::Schema;

use super::{parse::ParsedOperation, Operation};

pub(super) fn prepare_metrics_attributes(
    operation: &ParsedOperation,
    request: &engine::Request,
) -> Option<OperationMetricsAttributes> {
    operation_normalizer::normalize(request.query(), request.operation_name())
        .ok()
        .map(|sanitized_query| OperationMetricsAttributes {
            ty: operation.definition.ty.into(),
            name: operation.name.clone().or_else(|| {
                engine_parser::find_first_field_name(&operation.fragments, &operation.definition.selection_set)
            }),
            sanitized_query_hash: blake3::hash(sanitized_query.as_bytes()).into(),
            sanitized_query,
            // Added after the binding step
            used_fields: String::new(),
        })
}

pub(super) fn generate_used_fields(schema: &Schema, operation: &Operation) -> String {
    let mut used_field_definitions = Vec::with_capacity(operation.fields.len());
    for field in &operation.fields {
        let Some(definition_id) = field.definition_id() else {
            continue;
        };

        let field = schema.walk(definition_id);
        let entity = field.parent_entity();
        // Skipping introspection related fields
        if !entity.name().starts_with("__") && !field.name().starts_with("__") {
            used_field_definitions.push((entity.id(), definition_id))
        }
    }
    used_field_definitions.sort_unstable();

    // Kind of arbitrary, we may have duplicated fields but each field & type name will take
    // several bytes.
    let mut out = String::with_capacity(used_field_definitions.len() * 4);
    for (entity_id, field_definitions) in used_field_definitions
        .into_iter()
        .dedup()
        .chunk_by(|(entity_id, _)| *entity_id)
        .into_iter()
    {
        out.push_str(schema.walk(entity_id).name());
        out.push('.');
        for s in Itertools::intersperse(
            field_definitions.map(|(_, definition_id)| schema.walk(definition_id).name()),
            "+",
        ) {
            out.push_str(s);
        }
        out.push(',')
    }
    out.pop();

    out
}
