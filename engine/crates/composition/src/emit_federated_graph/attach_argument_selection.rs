use super::*;

/// Attach a selection set on the arguments of a field defined in strings to a FederatedGraph, transforming the strings into input value definition ids.
pub(super) fn attach_argument_selection(
    selection_set: &[subgraphs::Selection],
    field_id: federated::FieldId,
    ctx: &mut Context<'_>,
) -> federated::InputValueDefinitionSet {
    selection_set
        .iter()
        .map(|selection| {
            let selection_field = ctx.insert_string(ctx.subgraphs.walk(selection.field));
            let field_arguments = ctx.out[field_id].arguments;
            let argument_id = federated::InputValueDefinitionId::from(
                field_arguments.0 .0
                    + ctx.out[field_arguments]
                        .iter()
                        .position(|arg| arg.name == selection_field)
                        .unwrap(),
            );

            let subselection: federated::InputValueDefinitionSet =
                if let federated::Definition::InputObject(_) = ctx.out[argument_id].r#type.definition {
                    todo!("Nested selections not implement yet for arguments in @authorized: GB-6965")
                } else {
                    Vec::new()
                };

            federated::InputValueDefinitionSetItem {
                input_value_definition: argument_id,
                subselection,
            }
        })
        .collect()
}
