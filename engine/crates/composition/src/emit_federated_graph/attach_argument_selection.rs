use super::*;

/// Attach a selection set on the arguments of a field defined in strings to a FederatedGraph, transforming the strings into input value definition ids.
pub(super) fn attach_argument_selection(
    selection_set: &[subgraphs::Selection],
    field_id: federated::FieldId,
    ctx: &mut Context<'_>,
) -> federated::InputValueDefinitionSet {
    selection_set
        .iter()
        .filter_map(|selection| {
            let subgraphs::Selection::Field(subgraphs::FieldSelection {
                field,
                arguments: _,
                subselection,
            }) = selection
            else {
                return None;
            };

            let selection_field = ctx.insert_string(ctx.subgraphs.walk(*field));
            let argument_id = ctx
                .out
                .iter_field_arguments(field_id)
                .find(|arg| arg.name == selection_field)
                .unwrap()
                .id();

            let subselection: federated::InputValueDefinitionSet =
                if let federated::Definition::InputObject(input_object_id) = ctx.out[argument_id].r#type.definition {
                    attach_selection_on_input_object(subselection, input_object_id, ctx)
                } else {
                    Vec::new()
                };

            Some(federated::InputValueDefinitionSetItem {
                input_value_definition: argument_id,
                subselection,
            })
        })
        .collect()
}

fn attach_selection_on_input_object(
    selection_set: &[subgraphs::Selection],
    input_object_id: federated::TypeDefinitionId,
    ctx: &mut Context<'_>,
) -> federated::InputValueDefinitionSet {
    selection_set
        .iter()
        .filter_map(|selection| {
            let subgraphs::Selection::Field(subgraphs::FieldSelection {
                field, subselection, ..
            }) = selection
            else {
                return None;
            };

            let field_name = ctx.insert_string(ctx.subgraphs.walk(*field));

            let field_id = ctx
                .out
                .iter_input_object_fields(input_object_id)
                .find(|field| field.name == field_name)?
                .id();

            let subselection: federated::InputValueDefinitionSet =
                if let federated::Definition::InputObject(input_object_id) = ctx.out[field_id].r#type.definition {
                    attach_selection_on_input_object(subselection, input_object_id, ctx)
                } else {
                    Vec::new()
                };

            Some(federated::InputValueDefinitionSetItem {
                input_value_definition: field_id,
                subselection,
            })
        })
        .collect()
}
