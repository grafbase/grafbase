use crate::subgraphs;

mod input_selection;
mod subgraph_names;

type ValidateContext<'a> = crate::ComposeContext<'a>;

/// Pre-composition validations happen here.
pub(crate) fn validate(ctx: &mut ValidateContext<'_>) {
    subgraph_names::validate_subgraph_names(ctx);
    validate_query_nonempty(ctx);
    validate_fields(ctx);
}

fn validate_query_nonempty(ctx: &mut ValidateContext<'_>) {
    if ctx
        .subgraphs
        .iter_subgraphs()
        .filter_map(|subgraph| subgraph.query_type())
        .all(|query_type| query_type.fields().next().is_none())
    {
        ctx.diagnostics
            .push_fatal(String::from("None of the subgraphs defines root query fields."));
    }
}

fn validate_fields(ctx: &mut ValidateContext<'_>) {
    for field in ctx.subgraphs.iter_all_fields() {
        validate_selections(ctx, field);
        validate_override_labels(ctx, field);
        input_selection::validate_input_selections(ctx, field);
    }
}

fn validate_override_labels(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldWalker<'_>) {
    let Some(label) = field.directives().r#override().and_then(|directive| directive.label) else {
        return;
    };

    let Err(err) = ctx
        .subgraphs
        .walk(label)
        .as_str()
        .parse::<graphql_federated_graph::OverrideLabel>()
    else {
        return;
    };

    ctx.diagnostics.push_fatal(format!(
        "Invalid @override label argument on {ty}.{field}: {err}",
        ty = field.parent_definition().name().as_str(),
        field = field.name().as_str(),
    ));
}

fn validate_selections(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldWalker<'_>) {
    let directives = field.directives();
    for (selection, directive_name) in directives
        .requires()
        .into_iter()
        .flatten()
        .map(|selection| (selection, "requires"))
        .chain(
            directives
                .authorized()
                .into_iter()
                .flat_map(|auth| auth.fields.iter())
                .flatten()
                .map(|selection| (selection, "authorized")),
        )
    {
        let directive_path = || {
            format!(
                "{}.{}",
                field.parent_definition().name().as_str(),
                field.name().as_str()
            )
        };
        validate_selection(
            ctx,
            selection,
            field.parent_definition(),
            &directive_path,
            directive_name,
        );
    }
}

fn validate_selection(
    ctx: &mut ValidateContext<'_>,
    selection: &subgraphs::Selection,
    on_definition: subgraphs::DefinitionWalker<'_>,
    directive_path: &dyn Fn() -> String,
    directive_name: &str,
) {
    // The selected field must exist.
    let Some(field) = on_definition.find_field(selection.field) else {
        return ctx.diagnostics.push_fatal(format!(
            "Error in @{directive_name} at {directive_path}: the {field_in_selection} field does not exist on {definition_name}",
            field_in_selection = ctx.subgraphs.walk(selection.field).as_str(),
            directive_path = directive_path(),
            definition_name = on_definition.name().as_str()
        ));
    };

    for required_argument in field
        .arguments()
        .filter(|arg| arg.r#type().is_required() && arg.default().is_none())
    {
        let arg_name = required_argument.name();
        if selection.arguments.iter().all(|(name, _)| *name != arg_name.id) {
            ctx.diagnostics.push_fatal(format!(
                "Error in @{directive_name} on {directive_path}: the {field_name}.{arg_name} argument is required but not provided.",
                field_name = field.name().as_str(),
                arg_name = arg_name.as_str(),
                directive_path = directive_path(),
            ));
        }
    }

    // The arguments must exist on the field.
    for (argument_name, argument_value) in &selection.arguments {
        let Some(argument) = field.argument_by_name(*argument_name) else {
            return ctx.diagnostics.push_fatal(format!(
                "Error in @{directive_name} on {directive_path}: the {field_in_selection}.{argument_name} argument does not exist on {definition_name}",
                argument_name = ctx.subgraphs.walk(*argument_name).as_str(),
                field_in_selection = field.name().as_str(),
                definition_name = on_definition.name().as_str(),
                directive_path = directive_path(),
            ));
        };

        if !argument_type_matches(on_definition.subgraph_id(), argument.r#type(), argument_value) {
            return ctx.diagnostics.push_fatal(format!(
                "Error in @{directive_name} on {directive_path}: the {field_in_selection}.{argument_name} argument does not not match the expected type ({expected_type})",
                argument_name = ctx.subgraphs.walk(*argument_name).as_str(),
                field_in_selection = field.name().as_str(),
                expected_type = argument.r#type(),
                directive_path = directive_path(),
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
