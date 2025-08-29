use super::*;

/// The arguments of a federated graph's fields are the intersection of the subgraph's arguments for
/// that field.
pub(super) fn merge_field_arguments<'a>(
    first: subgraphs::FieldView<'a>,
    fields: &[subgraphs::FieldView<'a>],
    ctx: &mut Context<'a>,
) -> federated::InputValueDefinitions {
    let parent_definition = ctx.subgraphs.at(first.parent_definition_id);
    let field_name = first.name;
    let mut ids: Option<federated::InputValueDefinitions> = None;

    let intersection: HashSet<StringId> = first
        .arguments(ctx.subgraphs)
        .map(|arg| arg.name)
        .filter(|arg_name| {
            fields[1..]
                .iter()
                .all(|field| field.argument_by_name(ctx.subgraphs, *arg_name).is_some())
        })
        .collect();

    let mut all_arguments = fields
        .iter()
        .flat_map(|def| def.arguments(ctx.subgraphs))
        .collect::<Vec<_>>();

    all_arguments.sort_by_key(|arg| arg.name);

    let mut start = 0;

    while start < all_arguments.len() {
        let argument_name = all_arguments[start].name;
        let end = all_arguments[start..].partition_point(|arg| arg.name == argument_name) + start;
        let arguments = &all_arguments[start..end];

        start = end;

        let default = compose_field_argument_defaults(ctx, arguments).cloned();

        if !intersection.contains(&argument_name) {
            if let Some(required) = arguments.iter().find(|arg| arg.r#type.is_required()) {
                required_argument_not_in_intersection_error(ctx, fields, required);
            }

            continue;
        }

        let directive_containers = arguments.iter().map(|arg| arg.directives);
        let directives = collect_composed_directives(directive_containers, ctx);

        let Some(argument_type) =
            fields::compose_argument_types(parent_definition.name, arguments.iter().copied(), ctx)
        else {
            continue;
        };

        let argument_is_inaccessible = || arguments.iter().any(|arg| arg.directives.inaccessible(ctx.subgraphs));
        let argument_type_is_inaccessible = arguments.iter().any(|arg| {
            let parent_definition = arg.parent_definition_id;
            let subgraph_id = ctx.subgraphs.at(parent_definition).subgraph_id;
            let arg_type = ctx
                .subgraphs
                .definition_by_name_id(arg.r#type.definition_name_id, subgraph_id);

            arg_type
                .map(|def| ctx.subgraphs.at(def).directives.inaccessible(ctx.subgraphs))
                .unwrap_or(false)
        });

        if argument_type_is_inaccessible && !argument_is_inaccessible() {
            ctx.diagnostics.push_fatal(format!(
                "The argument `{}.{}({}:)` is of an @inaccessible type, but is itself not marked as @inaccessible.",
                ctx.subgraphs[parent_definition.name], ctx.subgraphs[field_name], ctx.subgraphs[argument_name],
            ));
        }

        let description = arguments
            .iter()
            .find_map(|arg| arg.description)
            .map(|description| ctx.insert_string(description));

        let name = ctx.insert_string(argument_name);
        let id = ctx.insert_input_value_definition(ir::InputValueDefinitionIr {
            name,
            r#type: argument_type,
            directives,
            description,
            default,
        });

        if let Some((_start, len)) = &mut ids {
            *len += 1;
        } else {
            ids = Some((id, 1));
        }
    }

    ids.unwrap_or(federated::NO_INPUT_VALUE_DEFINITION)
}

/// Default values on arguments (e.g. `field(arg: String! = "N.A")`) are _not_
/// present in the federated schema produced by composition. This function is only
/// here for validation.
///
/// The rule to enforce is that between the subgraphs that define a default
/// on the same fileld, the default must be the same. Other subgraphs can have the
/// same argument without default, that is valid, but everywhere a default value is
/// specified, it has to be the same.
fn compose_field_argument_defaults<'a>(
    ctx: &mut Context<'a>,
    arguments: &[subgraphs::ArgumentView<'a>],
) -> Option<&'a subgraphs::Value> {
    let mut default: Option<(&subgraphs::Value, subgraphs::ArgumentView<'_>)> = None;

    for argument in arguments {
        let Some(value) = argument.record.default_value.as_ref() else {
            continue;
        };

        match &mut default {
            None => {
                default = Some((value, *argument));
            }
            Some((default, _)) if default == &value => (),
            Some((_, other_argument)) => {
                let definition = ctx.subgraphs.at(argument.parent_definition_id);

                let first_subgraph_id = ctx.subgraphs.at(other_argument.parent_definition_id).subgraph_id;
                let first_subgraph = ctx.subgraphs.at(first_subgraph_id);
                let second_subgraph = ctx.subgraphs.at(definition.subgraph_id);

                ctx.diagnostics.push_fatal(format!(
                r#"The argument {type_name}.{field_name}.{argument_name} has incompatible defaults in subgraphs "{first_subgraph}" and "{second_subgraph}""#,
                type_name = ctx.subgraphs[definition.name],
                field_name = ctx.subgraphs[argument.parent_field_name],
                argument_name = ctx.subgraphs[argument.name],
                first_subgraph = ctx.subgraphs[first_subgraph.name],
                second_subgraph = ctx.subgraphs[second_subgraph.name],
            ))
            }
        }
    }

    default.map(|(default, _)| default)
}

fn required_argument_not_in_intersection_error(
    ctx: &mut Context<'_>,
    fields: &[subgraphs::FieldView<'_>],
    required_arg: &subgraphs::ArgumentRecord,
) {
    let definition_id_where_required = required_arg.parent_definition_id;
    let field_name = required_arg.parent_field_name;
    let argument_name = required_arg.name;

    let definition_where_required = ctx.subgraphs.at(definition_id_where_required);
    let subgraph_where_required = ctx.subgraphs.at(definition_where_required.subgraph_id);

    let subgraphs_where_missing = fields
        .iter()
        .filter(|field| field.argument_by_name(ctx.subgraphs, argument_name).is_none())
        .map(|field| {
            let def = ctx.subgraphs.at(field.parent_definition_id);
            let subgraph = ctx.subgraphs.at(def.subgraph_id);
            ctx.subgraphs[subgraph.name].as_ref()
        })
        .collect::<Vec<_>>();

    ctx.diagnostics.push_fatal(format!(
        "The argument `{}.{}({}:)` is required in {} but missing in {}.",
        ctx.subgraphs[ctx.subgraphs[definition_id_where_required].name],
        ctx.subgraphs[field_name],
        ctx.subgraphs[argument_name],
        ctx.subgraphs[subgraph_where_required.name],
        subgraphs_where_missing.join(", "),
    ));
}

pub(super) fn compose_fields<'a>(
    ctx: &mut Context<'a>,
    definitions: &[DefinitionView<'a>],
    parent_definition_name: federated::StringId,
) -> Vec<ir::FieldIr> {
    let mut field_irs = Vec::new();
    fields::for_each_field_group(ctx.subgraphs, definitions, |fields| {
        let Some(first) = fields.first() else { return };
        let Some(field) = compose_field(parent_definition_name, *first, fields, ctx) else {
            return;
        };
        field_irs.push(field)
    });
    field_irs
}

pub(super) fn compose_field<'a>(
    parent_definition_name: federated::StringId,
    first: subgraphs::FieldView<'a>,
    fields: &[subgraphs::FieldView<'a>],
    ctx: &mut Context<'a>,
) -> Option<ir::FieldIr> {
    if let DefinitionKind::Object = ctx.subgraphs.at(first.parent_definition_id).kind {
        crate::validate::composite_schemas::post_merge::invalid_field_sharing(ctx, fields);
    }

    if fields.iter().any(|field| {
        !field.directives.inaccessible(ctx.subgraphs)
            && ctx
                .subgraphs
                .definition_by_name_id(
                    field.r#type.definition_name_id,
                    ctx.subgraphs.at(field.parent_definition_id).subgraph_id,
                )
                .filter(|parent| ctx.subgraphs.at(*parent).directives.inaccessible(ctx.subgraphs))
                .is_some()
    }) {
        let name = format!(
            "{}.{}",
            ctx.subgraphs[ctx.subgraphs.at(first.parent_definition_id).name].as_ref(),
            ctx.subgraphs[first.name].as_ref()
        );
        let non_marked_subgraphs = fields
            .iter()
            .filter(|field| !field.directives.inaccessible(ctx.subgraphs));

        ctx.diagnostics.push_fatal(format!(
            "The field `{name}` is of an @inaccessible type, but is itself not marked as @inaccessible in subgraphs {}",
            non_marked_subgraphs
                .into_iter()
                .map(|f| {
                    let def = ctx.subgraphs.at(f.parent_definition_id);
                    ctx.subgraphs[ctx.subgraphs.at(def.subgraph_id).name].as_ref()
                })
                .join(", "),
        ));
    }

    let arguments = object::merge_field_arguments(first, fields, ctx);

    let field_type = fields::compose_output_field_types(ctx, fields.iter().copied())?;

    let mut directives = collect_composed_directives(fields.iter().map(|f| f.directives), ctx);
    ingest_join_field_directives(ctx, field_type, fields, &mut directives);

    let description = fields.iter().find_map(|f| f.description.map(|d| ctx.insert_string(d)));

    Some(ir::FieldIr {
        parent_definition_name,
        field_name: first.name,
        field_type,
        arguments,
        directives,
        description,
    })
}

fn ingest_join_field_directives(
    ctx: &mut Context<'_>,
    composed_field_type: subgraphs::FieldType,
    fields: &[subgraphs::FieldView<'_>],
    out: &mut Vec<ir::Directive>,
) {
    super::validate::override_source_has_override(fields, ctx);

    for field in fields {
        let directives = field.directives;
        let field_name = field.name;
        let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
        let is_external = directives.external(ctx.subgraphs) || parent_definition.directives.external(ctx.subgraphs);

        let mut directive = ir::JoinFieldDirective {
            source_field: field.id,
            r#override: None,
            override_label: None,
            external: is_external && !field.is_part_of_key(ctx.subgraphs),
            r#type: if field.r#type != composed_field_type {
                Some(field.r#type)
            } else {
                None
            },
        };

        if let Some(r#override) = field.directives.r#override(ctx.subgraphs) {
            directive.override_label = r#override.label.and_then(|label| ctx.subgraphs[label].parse().ok());
            directive.r#override = Some(
                ctx.subgraphs
                    .iter_subgraphs()
                    .position(|subgraph| subgraph.name == r#override.from)
                    .map(federated::SubgraphId::from)
                    .map(federated::OverrideSource::Subgraph)
                    .unwrap_or_else(|| federated::OverrideSource::Missing(ctx.insert_string(r#override.from))),
            );
        }

        if directives.requires(ctx.subgraphs).is_some() && is_external {
            let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
            ctx.diagnostics.push_fatal(format!(
                "field `{}` on `{}` declared as `@external` in subgraph `{}` cannot have a `@requires`.",
                ctx.subgraphs[field_name],
                ctx.subgraphs[parent_definition.name],
                ctx.subgraphs[ctx.subgraphs.at(parent_definition.subgraph_id).name],
            ));
        }

        if directives.provides(ctx.subgraphs).is_some() && is_external {
            let parent_definition = ctx.subgraphs.at(field.parent_definition_id);
            ctx.diagnostics.push_fatal(format!(
                "field `{}` on `{}` declared as `@external` in subgraph `{}` cannot have a `@provides`.",
                ctx.subgraphs[field_name],
                ctx.subgraphs[parent_definition.name],
                ctx.subgraphs[ctx.subgraphs.at(parent_definition.subgraph_id).name],
            ));
        }

        out.push(ir::Directive::JoinField(directive));
    }
}

pub(crate) fn validate_shareable_object_fields_match(definitions: &[DefinitionView<'_>], ctx: &mut ComposeContext<'_>) {
    let all_fields: BTreeSet<StringId> = definitions
        .iter()
        .flat_map(|def| def.id.fields(ctx.subgraphs))
        .map(|field| field.name)
        .collect();
    let inaccessible_fields: BTreeSet<StringId> = definitions
        .iter()
        .flat_map(|def| def.id.fields(ctx.subgraphs))
        .filter(|field| field.directives.inaccessible(ctx.subgraphs))
        .map(|field| field.name)
        .collect();

    for definition in definitions {
        for field in all_fields.difference(&inaccessible_fields) {
            if definition.id.field_by_name(ctx.subgraphs, *field).is_none() {
                ctx.diagnostics.push_fatal(format!(
                    "[{}] The shareable object `{}` is missing the `{}` field defined in other subgraphs.",
                    ctx.subgraphs[ctx.subgraphs.at(ctx.subgraphs.at(definition.id).subgraph_id).name],
                    ctx.subgraphs[definition.name],
                    ctx.subgraphs[*field],
                ));
            }
        }
    }
}
