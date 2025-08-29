use crate::diagnostics::CompositeSchemasSourceSchemaValidationErrorCode;

use super::*;

pub(super) fn validate_selections(ctx: &mut ValidateContext<'_>, field: subgraphs::FieldView<'_>) {
    let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
    let subgraph_name = &ctx.subgraphs[ctx.subgraphs[parent_definition.subgraph_id].name];

    for (selection, directive_name) in field
        .directives
        .requires(ctx.subgraphs)
        .into_iter()
        .flatten()
        .map(|selection| (selection, "requires"))
    {
        let directive_path = || {
            format!(
                "{}.{}",
                ctx.subgraphs[parent_definition.name], ctx.subgraphs[field.name]
            )
        };
        let parent_definition = ctx.subgraphs.at(field.parent_definition_id);

        validate_selection(
            ctx,
            selection,
            parent_definition,
            &directive_path,
            directive_name,
            subgraph_name,
        );
    }

    for selection in field.directives.provides(ctx.subgraphs).into_iter().flatten() {
        let directive_path = || {
            format!(
                "{}.{}",
                ctx.subgraphs[parent_definition.name], ctx.subgraphs[field.name]
            )
        };

        let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
        let referenced_definition = ctx
            .subgraphs
            .definition_by_name_id(field.r#type.definition_name_id, parent_definition.subgraph_id);

        let Some(field_type) = referenced_definition else {
            ctx.diagnostics.push_fatal(format!(
                "Invalid @provides at {}: no selection possible on this field type.",
                directive_path()
            ));
            continue;
        };

        validate_selection_in_provides(ctx, selection, &directive_path, subgraph_name);

        let field_type = ctx.subgraphs.at(field_type);
        validate_selection(ctx, selection, field_type, &directive_path, "provides", subgraph_name);
    }
}

fn validate_selection_in_provides(
    ctx: &mut ValidateContext<'_>,
    selection: &subgraphs::Selection,
    directive_path: &dyn Fn() -> String,
    subgraph_name: &str,
) {
    match selection {
        subgraphs::Selection::Field(subgraphs::FieldSelection {
            field: _,
            arguments: _,
            subselection: _,
            has_directives,
        })
        | subgraphs::Selection::InlineFragment {
            on: _,
            subselection: _,
            has_directives,
        } if *has_directives => {
            ctx.diagnostics.push_composite_schemas_source_schema_validation_error(
                subgraph_name,
                format!(
                    "Error at {}: no directives are allowed in the selection sets in `@provides(fields:)`.",
                    directive_path()
                ),
                CompositeSchemasSourceSchemaValidationErrorCode::ProvidesDirectiveInFieldsArgument,
            );
        }
        subgraphs::Selection::Field(subgraphs::FieldSelection {
            field: _,
            arguments: _,
            subselection,
            has_directives: _,
        })
        | subgraphs::Selection::InlineFragment {
            on: _,
            subselection,
            has_directives: _,
        } => {
            for selection in subselection {
                validate_selection_in_provides(ctx, selection, directive_path, subgraph_name);
            }
        }
    }
}

fn validate_selection(
    ctx: &mut ValidateContext<'_>,
    selection: &subgraphs::Selection,
    on_definition: subgraphs::View<'_, subgraphs::DefinitionId, subgraphs::Definition>,
    directive_path: &dyn Fn() -> String,
    directive_name: &str,
    subgraph_name: &str,
) {
    match selection {
        subgraphs::Selection::Field(field_selection) => validate_field_selection(
            ctx,
            field_selection,
            on_definition,
            directive_path,
            directive_name,
            subgraph_name,
        ),
        subgraphs::Selection::InlineFragment {
            on,
            subselection,
            has_directives: _,
        } => {
            let Some(on) = ctx.subgraphs.definition_by_name_id(*on, on_definition.subgraph_id) else {
                let directive_path = directive_path();
                let on = &ctx.subgraphs[*on];
                ctx.diagnostics.push_fatal(format!(
                    "[{subgraph_name}] Error in {directive_name} at {directive_path}: type condition `... {on}` is invalid on {parent_definition}",
                    parent_definition = ctx.subgraphs[on_definition.name]
                ));
                return;
            };

            for selection in subselection {
                validate_selection(
                    ctx,
                    selection,
                    ctx.subgraphs.at(on),
                    directive_path,
                    directive_name,
                    subgraph_name,
                );
            }
        }
    }
}

fn validate_field_selection(
    ctx: &mut ValidateContext<'_>,
    selection: &subgraphs::FieldSelection,
    on_definition: subgraphs::View<'_, subgraphs::DefinitionId, subgraphs::Definition>,
    directive_path: &dyn Fn() -> String,
    directive_name: &str,
    subgraph_name: &str,
) {
    if &ctx[selection.field] == "__typename" {
        if !selection.arguments.is_empty() {
            return ctx.diagnostics.push_fatal(format!(
                "[{subgraph_name}] Error in @{directive_name} on {directive_path}: the __typename field does not accept arguments.",
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
    let Some(field) = on_definition.id.field_by_name(ctx.subgraphs, selection.field) else {
        return ctx.diagnostics.push_fatal(format!(
            "[{subgraph_name}] Error in @{directive_name} at {directive_path}: the {field_in_selection} field does not exist on {definition_name}",
            field_in_selection = ctx.subgraphs[selection.field],
            directive_path = directive_path(),
            definition_name = ctx.subgraphs[on_definition.name]
        ));
    };

    for required_argument in field
        .arguments(ctx.subgraphs)
        .filter(|arg| arg.r#type.is_required() && arg.default_value.is_none())
    {
        let arg_name = required_argument.name;
        if selection.arguments.iter().all(|(name, _)| *name != arg_name) {
            ctx.diagnostics.push_fatal(format!(
                "[{subgraph_name}] Error in @{directive_name} on {directive_path}: the {field_name}.{arg_name} argument is required but not provided.",
                field_name = ctx.subgraphs[field.name],
                arg_name = ctx.subgraphs[arg_name],
                directive_path = directive_path(),
            ));
        }
    }

    // The arguments must exist on the field.
    for (argument_name, argument_value) in &selection.arguments {
        let Some(argument) = field.argument_by_name(ctx.subgraphs, *argument_name) else {
            return ctx.diagnostics.push_fatal(format!(
                "[{subgraph_name}] Error in @{directive_name} on {directive_path}: the {field_in_selection}.{argument_name} argument does not exist on {definition_name}",
                argument_name = ctx.subgraphs[*argument_name],
                field_in_selection = ctx.subgraphs[field.name],
                definition_name = ctx.subgraphs[on_definition.name],
                directive_path = directive_path(),
            ));
        };

        if !argument_type_matches(ctx, on_definition.subgraph_id, &argument.r#type, argument_value) {
            return ctx.diagnostics.push_fatal(format!(
                "[{subgraph_name}] Error in @{directive_name} on {directive_path}: the {field_in_selection}.{argument_name} argument does not not match the expected type ({expected_type})",
                argument_name = ctx.subgraphs[*argument_name],
                field_in_selection = ctx.subgraphs[field.name],
                expected_type = argument.r#type.display(ctx.subgraphs),
                directive_path = directive_path(),
            ));
        }
    }

    if selection.subselection.is_empty() {
        return;
    }

    let referenced_type = ctx
        .subgraphs
        .definition_by_name_id(field.r#type.definition_name_id, on_definition.subgraph_id)
        .expect("type is defined in subgraph");

    for selection in &selection.subselection {
        validate_selection(
            ctx,
            selection,
            ctx.subgraphs.at(referenced_type),
            directive_path,
            directive_name,
            subgraph_name,
        );
    }
}

fn argument_type_matches(
    ctx: &mut ValidateContext<'_>,
    subgraph: subgraphs::SubgraphId,
    arg_type: &subgraphs::FieldType,
    value: &subgraphs::Value,
) -> bool {
    let arg_type_name = ctx.subgraphs[arg_type.definition_name_id].as_ref();

    match value {
        subgraphs::Value::String(_) if arg_type_name == "String" => true,
        subgraphs::Value::Int(_) if arg_type_name == "Int" => true,
        subgraphs::Value::Float(_) if arg_type_name == "Float" => true,
        subgraphs::Value::Boolean(_) if arg_type_name == "Boolean" => true,
        subgraphs::Value::Enum(value) => {
            let Some(enum_type) = ctx
                .subgraphs
                .definition_by_name_id(arg_type.definition_name_id, subgraph)
            else {
                return false;
            };

            enum_type.enum_value_by_name(ctx.subgraphs, *value).is_some()
        }
        subgraphs::Value::Object(fields) => {
            let Some(input_object_type) = ctx
                .subgraphs
                .definition_by_name_id(arg_type.definition_name_id, subgraph)
            else {
                return false;
            };

            fields.iter().all(|(field_name, field_value)| {
                let Some(inner_field) = input_object_type.field_by_name(ctx.subgraphs, *field_name) else {
                    return false;
                };

                argument_type_matches(ctx, subgraph, &inner_field.r#type, field_value)
            })
        }
        subgraphs::Value::List(inner) if arg_type.is_list() => inner
            .iter()
            .all(|inner| argument_type_matches(ctx, subgraph, arg_type, inner)),
        _ => false,
    }
}

pub(crate) fn validate_keys(ctx: &mut ValidateContext<'_>) {
    for key in ctx.subgraphs.iter_keys() {
        let directive_path = || {
            let definition = &ctx.subgraphs[key.definition_id];
            ctx.subgraphs[definition.name].to_string()
        };

        let parent_definition = ctx.subgraphs.at(key.definition_id);
        let subgraph = ctx.subgraphs.at(parent_definition.subgraph_id);
        let subgraph_name = &ctx.subgraphs[subgraph.name];

        for selection in &key.selection_set {
            validate_selection(ctx, selection, parent_definition, &directive_path, "key", subgraph_name)
        }
    }
}
