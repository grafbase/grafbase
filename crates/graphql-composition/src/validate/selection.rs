use super::*;

pub(super) fn validate_selections(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldWalker<'_>) {
    let directives = field.directives();
    for (selection, directive_name) in directives
        .requires()
        .into_iter()
        .flatten()
        .map(|selection| (selection, "requires"))
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

    for selection in directives.provides().into_iter().flatten() {
        let directive_path = || {
            format!(
                "{}.{}",
                field.parent_definition().name().as_str(),
                field.name().as_str()
            )
        };

        let field_type = field
            .r#type()
            .definition(field.parent_definition().subgraph_id())
            .unwrap();

        validate_selection(ctx, selection, field_type, &directive_path, "provides");
    }
}

fn validate_selection(
    ctx: &mut ValidateContext<'_>,
    selection: &subgraphs::Selection,
    on_definition: subgraphs::DefinitionWalker<'_>,
    directive_path: &dyn Fn() -> String,
    directive_name: &str,
) {
    match selection {
        subgraphs::Selection::Field(field_selection) => {
            validate_field_selection(ctx, field_selection, on_definition, directive_path, directive_name)
        }
        subgraphs::Selection::InlineFragment { on, subselection } => {
            let subgraph_id = on_definition.subgraph_id();
            let Some(on) = ctx.subgraphs.definition_by_name_id(*on, subgraph_id) else {
                let directive_path = directive_path();
                ctx.diagnostics.push_fatal(format!(
                    "Error in {directive_name} at {directive_path}: type condition `... {on}` is invalid on {parent_definition}",
                    on = ctx.subgraphs.walk(*on).as_str(),
                    parent_definition = on_definition.name().as_str()
                ));
                return;
            };

            for selection in subselection {
                validate_selection(ctx, selection, ctx.subgraphs.walk(on), directive_path, directive_name);
            }
        }
    }
}

fn validate_field_selection(
    ctx: &mut ValidateContext<'_>,
    selection: &subgraphs::FieldSelection,
    on_definition: subgraphs::DefinitionWalker<'_>,
    directive_path: &dyn Fn() -> String,
    directive_name: &str,
) {
    if &ctx[selection.field] == "__typename" {
        if !selection.arguments.is_empty() {
            return ctx.diagnostics.push_fatal(format!(
                "Error in @{directive_name} on {directive_path}: the __typename field does not accept arguments.",
                directive_path = directive_path(),
            ));
        }
        if !selection.subselection.is_empty() {
            return ctx.diagnostics.push_fatal(format!(
                "Error in @{directive_name} on {directive_path}: the __typename field does not accept subselections.",
                directive_path = directive_path(),
            ));
        }
        return;
    }
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

    for selection in &selection.subselection {
        validate_selection(
            ctx,
            selection,
            field
                .r#type()
                .definition(on_definition.subgraph_id())
                .expect("type is defined in subgraph"),
            directive_path,
            directive_name,
        );
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

pub(crate) fn validate_keys(ctx: &mut ValidateContext<'_>) {
    for key in ctx.subgraphs.iter_keys() {
        let directive_path = || {
            let definition = &ctx.subgraphs[key.definition_id];
            ctx.subgraphs[definition.name].to_string()
        };

        for selection in &key.selection_set {
            validate_selection(
                ctx,
                selection,
                ctx.subgraphs.walk(key.definition_id),
                &directive_path,
                "key",
            )
        }
    }
}
