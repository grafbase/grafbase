use subgraphs::FieldTypeWalker;

use super::*;

/// The arguments of a federated graph's fields are the intersection of the subgraph's arguments for
/// that field.
pub(super) fn merge_field_arguments<'a>(
    first: FieldWalker<'a>,
    fields: &[FieldWalker<'a>],
    ctx: &mut Context<'a>,
) -> federated::InputValueDefinitions {
    let parent_definition_name = first.parent_definition().name().id;
    let field_name = first.name().id;
    let mut ids: Option<federated::InputValueDefinitions> = None;

    let intersection: HashSet<StringId> = first
        .arguments()
        .map(|arg| arg.name().id)
        .filter(|arg_name| fields[1..].iter().all(|def| def.argument_by_name(*arg_name).is_some()))
        .collect();

    let mut all_arguments = fields
        .iter()
        .flat_map(|def| def.arguments())
        .map(|arg| (arg.name().id, arg))
        .collect::<Vec<_>>();

    all_arguments.sort_by_key(|(name, _)| *name);

    let mut start = 0;

    while start < all_arguments.len() {
        let argument_name = all_arguments[start].0;
        let end = all_arguments[start..].partition_point(|(name, _)| *name == argument_name) + start;
        let arguments = &all_arguments[start..end];

        start = end;

        let default = compose_field_argument_defaults(arguments, ctx).cloned();

        if !intersection.contains(&argument_name) {
            if let Some((_, required)) = arguments.iter().find(|(_name, arg)| arg.r#type().is_required()) {
                required_argument_not_in_intersection_error(
                    fields,
                    *required,
                    parent_definition_name,
                    field_name,
                    argument_name,
                    ctx,
                );
            }

            continue;
        }

        let directive_containers = arguments.iter().map(|(_, arg)| arg.directives());
        let directives = collect_composed_directives(directive_containers, ctx);

        let Some(argument_type) = fields::compose_argument_types(
            parent_definition_name,
            field_name,
            arguments.iter().map(|(_, arg)| *arg),
            ctx,
        ) else {
            continue;
        };

        let argument_is_inaccessible = || arguments.iter().any(|(_, arg)| arg.directives().inaccessible());
        let argument_type_is_inaccessible = arguments.iter().any(|(_, arg)| {
            arg.r#type()
                .definition(arg.field().parent_definition().subgraph_id())
                .map(|def| def.directives().inaccessible())
                .unwrap_or(false)
        });

        if argument_type_is_inaccessible && !argument_is_inaccessible() {
            ctx.diagnostics.push_fatal(format!(
                "The argument `{}.{}({}:)` is of an @inaccessible type, but is itself not marked as @inaccessible.",
                ctx.subgraphs.walk(parent_definition_name).as_str(),
                ctx.subgraphs.walk(field_name).as_str(),
                ctx.subgraphs.walk(argument_name).as_str(),
            ));
        }

        let description = arguments
            .iter()
            .find_map(|(_, arg)| arg.description())
            .map(|description| ctx.insert_string(description.id));

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
    arguments: &[(StringId, subgraphs::FieldArgumentWalker<'a>)],
    ctx: &mut Context<'a>,
) -> Option<&'a subgraphs::Value> {
    let mut default: Option<(&subgraphs::Value, subgraphs::FieldArgumentWalker<'_>)> = None;

    for (_, argument) in arguments {
        let Some(value) = argument.default() else { continue };

        match &mut default {
            None => {
                default = Some((value, *argument));
            }
            Some((default, _)) if default == &value => (),
            Some((_, other_argument)) => ctx.diagnostics.push_fatal(format!(
                r#"The argument {type_name}.{field_name}.{argument_name} has incompatible defaults in subgraphs "{first_subgraph}" and "{second_subgraph}""#,
                type_name = argument.field().parent_definition().name().as_str(),
                field_name = argument.field().name().as_str(),
                argument_name = argument.name().as_str(),
                first_subgraph = other_argument.field().parent_definition().subgraph().name().as_str(),
                second_subgraph = argument.field().parent_definition().subgraph().name().as_str(),
            )),
        }
    }

    default.map(|(default, _)| default)
}

fn required_argument_not_in_intersection_error(
    fields: &[FieldWalker<'_>],
    required_arg: subgraphs::FieldArgumentWalker<'_>,
    parent_definition_name: StringId,
    field_name: StringId,
    argument_name: StringId,
    ctx: &mut Context<'_>,
) {
    let subgraph_where_required = required_arg.field().parent_definition().subgraph().name().as_str();
    let subgraphs_where_missing = fields
        .iter()
        .filter(|field| field.argument_by_name(argument_name).is_none())
        .map(|field| field.parent_definition().subgraph().name().as_str())
        .collect::<Vec<_>>();
    ctx.diagnostics.push_fatal(format!(
        "The argument `{}.{}({}:)` is required in {} but missing in {}.",
        ctx.subgraphs.walk(parent_definition_name).as_str(),
        ctx.subgraphs.walk(field_name).as_str(),
        ctx.subgraphs.walk(argument_name).as_str(),
        subgraph_where_required,
        subgraphs_where_missing.join(", "),
    ));
}

pub(super) fn compose_fields<'a>(
    ctx: &mut Context<'a>,
    definitions: &[DefinitionWalker<'a>],
    parent_definition_name: federated::StringId,
) -> Vec<ir::FieldIr> {
    let mut field_irs = Vec::new();
    fields::for_each_field_group(definitions, |fields| {
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
    first: FieldWalker<'a>,
    fields: &[FieldWalker<'a>],
    ctx: &mut Context<'a>,
) -> Option<ir::FieldIr> {
    let field_name = first.name();

    if let DefinitionKind::Object = first.parent_definition().kind() {
        crate::validate::composite_schemas::post_merge::invalid_field_sharing(ctx, fields);
    }

    if fields.iter().any(|field| {
        !field.directives().inaccessible()
            && field
                .r#type()
                .definition(field.parent_definition().subgraph_id())
                .filter(|parent| parent.directives().inaccessible())
                .is_some()
    }) {
        let name = format!(
            "{}.{}",
            first.parent_definition().name().as_str(),
            first.name().as_str()
        );
        let non_marked_subgraphs = fields.iter().filter(|field| !field.directives().inaccessible());

        ctx.diagnostics.push_fatal(format!(
            "The field `{name}` is of an @inaccessible type, but is itself not marked as @inaccessible in subgraphs {}",
            non_marked_subgraphs
                .into_iter()
                .map(|f| f.parent_definition().subgraph().name().as_str())
                .join(", "),
        ));
    }

    let arguments = object::merge_field_arguments(first, fields, ctx);

    let field_name = ctx.insert_string(field_name.id);
    let field_type = fields::compose_output_field_types(fields.iter().copied(), ctx)?;

    let mut directives = collect_composed_directives(fields.iter().map(|f| f.directives()), ctx);
    ingest_join_field_directives(ctx, first.walk(field_type), fields, &mut directives);

    let description = fields
        .iter()
        .find_map(|f| f.description().map(|d| ctx.insert_string(d.id)));

    Some(ir::FieldIr {
        parent_definition_name,
        field_name,
        field_type,
        arguments,
        directives,
        description,
    })
}

fn ingest_join_field_directives(
    ctx: &mut Context<'_>,
    composed_field_type: FieldTypeWalker<'_>,
    fields: &[FieldWalker<'_>],
    out: &mut Vec<ir::Directive>,
) {
    super::validate::override_source_has_override(fields, ctx);

    for field in fields {
        let mut directive = ir::JoinFieldDirective {
            source_field: field.id,
            r#override: None,
            override_label: None,
            external: field.is_external() && !field.is_part_of_key(),
            r#type: if field.r#type() != composed_field_type {
                Some(field.r#type().id)
            } else {
                None
            },
        };

        if let Some(r#override) = field.directives().r#override() {
            directive.override_label = r#override
                .label
                .and_then(|label| ctx.subgraphs.walk(label).as_str().parse().ok());
            directive.r#override = Some(
                ctx.subgraphs
                    .iter_subgraphs()
                    .position(|subgraph| subgraph.name().id == r#override.from)
                    .map(federated::SubgraphId::from)
                    .map(federated::OverrideSource::Subgraph)
                    .unwrap_or_else(|| federated::OverrideSource::Missing(ctx.insert_string(r#override.from))),
            );
        }

        if field.directives().requires().is_some() && field.is_external() {
            ctx.diagnostics.push_fatal(format!(
                "field `{}` on `{}` declared as `@external` in subgraph `{}` cannot have a `@requires`.",
                field.name().as_str(),
                field.parent_definition().name().as_str(),
                field.parent_definition().subgraph().name().as_str(),
            ));
        }

        if field.directives().provides().is_some() && field.is_external() {
            ctx.diagnostics.push_fatal(format!(
                "field `{}` on `{}` declared as `@external` in subgraph `{}` cannot have a `@provides`.",
                field.name().as_str(),
                field.parent_definition().name().as_str(),
                field.parent_definition().subgraph().name().as_str(),
            ));
        }

        out.push(ir::Directive::JoinField(directive));
    }
}

pub(crate) fn validate_shareable_object_fields_match(
    definitions: &[DefinitionWalker<'_>],
    ctx: &mut ComposeContext<'_>,
) {
    let all_fields: BTreeSet<StringId> = definitions
        .iter()
        .flat_map(|def| def.fields())
        .map(|field| field.name().id)
        .collect();
    let inaccessible_fields: BTreeSet<StringId> = definitions
        .iter()
        .flat_map(|def| def.fields())
        .filter(|field| field.directives().inaccessible())
        .map(|field| field.name().id)
        .collect();

    for definition in definitions {
        for field in all_fields.difference(&inaccessible_fields) {
            if definition.find_field(*field).is_none() {
                ctx.diagnostics.push_fatal(format!(
                    "[{}] The shareable object `{}` is missing the `{}` field defined in other subgraphs.",
                    definition.subgraph().name().as_str(),
                    definition.name().as_str(),
                    definition.walk(*field).as_str(),
                ));
            }
        }
    }
}
