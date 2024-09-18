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
            let field_arguments = ctx.out[field_id].arguments;
            let argument_id = federated::InputValueDefinitionId::from(
                field_arguments.0 .0
                    + ctx.out[field_arguments]
                        .iter()
                        .position(|arg| arg.name == selection_field)
                        .unwrap(),
            );

            let subselection: federated::InputValueDefinitionSet =
                if let federated::Definition::InputObject(input_object_id) = ctx.out[argument_id].r#type.definition {
                    attach_selection_on_input_object(&subselection, input_object_id, ctx)
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
    input_object_id: federated::InputObjectId,
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
            let input_object = &ctx.out[input_object_id];
            let fields = &ctx.out[input_object.fields];

            let field_idx = fields.iter().position(|field| field.name == field_name)?;
            let field_id = federated::InputValueDefinitionId(input_object.fields.0 .0 + field_idx);

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
