use crate::subgraphs;

type ValidateContext<'a> = crate::ComposeContext<'a>;

/// Pre-composition validations happen here.
pub(crate) fn validate(ctx: &mut ValidateContext<'_>) {
    validate_selections(ctx);
}

fn validate_selections(ctx: &mut ValidateContext<'_>) {
    for field in ctx.subgraphs.iter_all_fields() {
        for selection in field.directives().requires().into_iter().flatten() {
            validate_selection(ctx, selection, field.parent_definition())
        }
    }
}

fn validate_selection(
    ctx: &mut ValidateContext<'_>,
    selection: &subgraphs::Selection,
    on_definition: subgraphs::DefinitionWalker<'_>,
) {
    // The selected field must exist.
    let Some(field) = on_definition.find_field(selection.field) else {
        return ctx.diagnostics.push_fatal(format!(
            "Error in @requires: the {field_in_selection} field does not exist on {definition_name}",
            field_in_selection = ctx.subgraphs.walk(selection.field).as_str(),
            definition_name = on_definition.name().as_str()
        ));
    };

    // The arguments must exist on the field.
    for (argument_name, argument_value) in &selection.arguments {
        let Some(argument) = field.argument_by_name(*argument_name) else {
            return ctx.diagnostics.push_fatal(format!(
                "Error in @requires: the {field_in_selection}.{argument_name} argument does not exist on {definition_name}",
                argument_name = ctx.subgraphs.walk(*argument_name).as_str(),
                field_in_selection = field.name().as_str(),
                definition_name = on_definition.name().as_str()
            ));
        };

        if !argument_type_matches(on_definition.subgraph_id(), argument.r#type(), argument_value) {
            return ctx.diagnostics.push_fatal(format!(
                "Error in @requires: the {field_in_selection}.{argument_name} argument does not not match the expected type ({expected_type})",
                argument_name = ctx.subgraphs.walk(*argument_name).as_str(),
                field_in_selection = field.name().as_str(),
                expected_type = argument.r#type(),
            ));
        }
    }
}

fn argument_type_matches(
    subgraph: subgraphs::SubgraphId,
    arg_type: subgraphs::FieldTypeWalker<'_>,
    value: &subgraphs::Value,
) -> bool {
    let arg_type_name = arg_type.type_name().as_str();

    match value {
        subgraphs::Value::String(_) if arg_type_name == "String" => true,
        subgraphs::Value::Int(_) if arg_type_name == "Int" => true,
        subgraphs::Value::Float(_) if arg_type_name == "Float" => true,
        subgraphs::Value::Boolean(_) if arg_type_name == "Boolean" => true,
        subgraphs::Value::Enum(value) => {
            let Some(enum_type) = arg_type.definition(subgraph) else {
                return false;
            };

            enum_type.enum_value_by_name(*value).is_some()
        }
        subgraphs::Value::Object(fields) => {
            let Some(input_object_type) = arg_type.definition(subgraph) else {
                return false;
            };

            fields.iter().all(|(field_name, field_value)| {
                let Some(inner_field) = input_object_type.find_field(*field_name) else {
                    return false;
                };

                argument_type_matches(subgraph, inner_field.r#type(), field_value)
            })
        }
        subgraphs::Value::List(inner) if arg_type.is_list() => inner
            .iter()
            .all(|inner| argument_type_matches(subgraph, arg_type, inner)),
        _ => false,
    }
}
