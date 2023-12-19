use crate::subgraphs::*;

/// See [crate::subgraphs::Keys::nested_key_fields].
pub(super) fn ingest_nested_key_fields(subgraph_id: SubgraphId, subgraphs: &mut Subgraphs) {
    subgraphs.with_nested_key_fields(|subgraphs, nested_key_fields| {
        for definition in subgraphs.walk_subgraph(subgraph_id).definitions() {
            for key in definition.entity_keys() {
                for field in key.fields() {
                    let Some(field_type) = definition
                        .find_field(field.field)
                        .and_then(|field| field.r#type().definition(subgraph_id))
                    else {
                        continue;
                    };

                    for subselection_field in &field.subselection {
                        ingest_nested_key_fields_rec(field_type, subselection_field, nested_key_fields);
                    }
                }
            }
        }
    });
}

fn ingest_nested_key_fields_rec(
    parent_definition: DefinitionWalker<'_>,
    selection: &Selection,
    nested_key_fields: &mut NestedKeyFields,
) {
    let Some(field) = parent_definition.find_field(selection.field) else {
        return;
    };

    nested_key_fields.insert(field);

    if selection.subselection.is_empty() {
        return;
    }

    let Some(selection_field_type) = field.r#type().definition(parent_definition.subgraph_id()) else {
        return;
    };

    for subselection in &selection.subselection {
        ingest_nested_key_fields_rec(selection_field_type, subselection, nested_key_fields);
    }
}
