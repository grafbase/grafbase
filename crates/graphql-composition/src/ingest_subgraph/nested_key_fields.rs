use super::Context;
use crate::subgraphs::*;

/// See [crate::subgraphs::Keys::nested_key_fields].
pub(super) fn ingest_nested_key_fields(ctx: &mut Context<'_>) {
    let subgraph_id = ctx.subgraph_id;
    ctx.subgraphs.with_nested_key_fields(|subgraphs, nested_key_fields| {
        for definition in subgraph_id.definitions(subgraphs) {
            for key in definition.id.keys(subgraphs) {
                for selection in &key.selection_set {
                    let Selection::Field(field) = selection else { continue };

                    let Some(field_type) = definition.id.field_by_name(subgraphs, field.field).and_then(|field| {
                        subgraphs.definition_by_name_id(field.r#type.definition_name_id, subgraph_id)
                    }) else {
                        continue;
                    };

                    for subselection_field in &field.subselection {
                        ingest_nested_key_fields_rec(subgraphs, field_type, subselection_field, nested_key_fields);
                    }
                }
            }
        }
    });
}

fn ingest_nested_key_fields_rec(
    subgraphs: &Subgraphs,
    parent_definition: DefinitionId,
    selection: &Selection,
    nested_key_fields: &mut NestedKeyFields,
) {
    let Selection::Field(FieldSelection {
        field,
        arguments: _,
        subselection,
        has_directives: _,
    }) = selection
    else {
        return;
    };

    let Some(field) = parent_definition.field_by_name(subgraphs, *field) else {
        return;
    };

    nested_key_fields.insert(field.record);

    if subselection.is_empty() {
        return;
    }

    let Some(selection_field_type) = subgraphs.definition_by_name_id(
        field.r#type.definition_name_id,
        subgraphs.at(parent_definition).subgraph_id,
    ) else {
        return;
    };

    for subselection in subselection {
        ingest_nested_key_fields_rec(subgraphs, selection_field_type, subselection, nested_key_fields);
    }
}
