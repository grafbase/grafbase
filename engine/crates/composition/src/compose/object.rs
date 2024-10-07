use super::*;

/// The arguments of a federated graph's fields are the interseciton of the subgraph's arguments for
/// that field. Returns (arg_name, arg_type, is_inaccessible).
pub(super) fn merge_field_arguments<'a>(
    first: FieldWalker<'a>,
    fields: &[FieldWalker<'a>],
    federated_field_id: federated::FieldId,
    ctx: &mut Context<'a>,
) {
    let parent_definition_name = first.parent_definition().name().id;
    let field_name = first.name().id;

    // We want to take the intersection of the field sets.
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
        let composed_directives = collect_composed_directives(directive_containers, ctx);

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

        let name = ctx.insert_string(argument_name);
        ctx.insert_input_value_definition(ir::InputValueDefinitionIr {
            location: federated_field_id.into(),
            name,
            r#type: argument_type,
            directives: composed_directives,
            description: None,
            default,
        });
    }
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

pub(super) fn compose_object_fields<'a>(
    parent_definition: federated::ObjectId,
    object_is_shareable: bool,
    first: FieldWalker<'a>,
    fields: &[FieldWalker<'a>],
    ctx: &mut Context<'a>,
) {
    let field_name = first.name();

    if !object_is_shareable
        && fields
            .iter()
            .filter(|f| {
                let d = f.directives();
                !(d.shareable() || d.external() || f.is_part_of_key() || d.r#override().is_some())
            })
            .count()
            > 1
    {
        let next = &fields[1];

        ctx.diagnostics.push_fatal(format!(
            "The field `{}` on `{}` is defined in two subgraphs (`{}` and `{}`).",
            first.name().as_str(),
            first.parent_definition().name().as_str(),
            first.parent_definition().subgraph().name().as_str(),
            next.parent_definition().subgraph().name().as_str(),
        ));
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

    let resolvable_in = resolvable_in(fields, object_is_shareable);

    let provides = fields
        .iter()
        .filter(|f| f.directives().provides().is_some())
        .map(|f| f.id.0)
        .collect();

    let requires = fields
        .iter()
        .filter(|f| f.directives().requires().is_some())
        .map(|f| f.id.0)
        .collect();

    let authorized_directives = fields
        .iter()
        .filter(|f| f.directives().authorized().is_some())
        .map(|field| field.id.0)
        .collect();

    let overrides = collect_overrides(fields, ctx);

    let description = fields
        .iter()
        .find_map(|f| f.description().map(|d| ctx.insert_string(d.id)));

    let composed_directives = collect_composed_directives(fields.iter().map(|f| f.directives()), ctx);

    let Some(field_type) = fields::compose_output_field_types(fields.iter().copied(), ctx) else {
        return;
    };

    let field_id = ctx.insert_field(ir::FieldIr {
        parent_definition: federated::Definition::Object(parent_definition),
        field_name: field_name.id,
        field_type,
        resolvable_in,
        provides,
        requires,
        composed_directives,
        overrides,
        description,
        authorized_directives,
    });

    object::merge_field_arguments(first, fields, field_id, ctx);
}

fn resolvable_in(fields: &[FieldWalker<'_>], object_is_shareable: bool) -> Vec<federated::SubgraphId> {
    if object_is_shareable || fields.iter().any(|f| f.directives().r#override().is_some()) {
        return vec![];
    }

    fields
        .iter()
        .filter(|field| !field.directives().external() && !field.is_part_of_key())
        .map(|field| federated::SubgraphId(field.parent_definition().subgraph_id().idx()))
        .collect()
}

pub(super) fn collect_overrides(fields: &[FieldWalker<'_>], ctx: &mut Context<'_>) -> Vec<federated::Override> {
    let mut overrides = Vec::new();

    for (field, override_directive) in fields.iter().filter_map(|f| Some(f).zip(f.directives().r#override())) {
        let field_subgraph = field.parent_definition().subgraph();

        if override_directive.from == field_subgraph.name().id {
            ctx.diagnostics.push_fatal(format!(
                r#"Source and destination subgraphs "{}" are the same for overridden field "{}.{}""#,
                ctx.subgraphs.walk(override_directive.from).as_str(),
                field.parent_definition().name().as_str(),
                field.name().as_str()
            ));
            continue;
        }

        if let Some(override_source) = fields
            .iter()
            .find(|f| f.parent_definition().subgraph().name().id == override_directive.from)
        {
            if override_source.directives().r#override().is_some() {
                ctx.diagnostics
                    .push_fatal(format!(r#"Field "{}.{}" on subgraph "{}" is also marked with directive @override in subgraph "{}". Only one @override directive is allowed per field."#,
                        override_source.parent_definition().name().as_str(),
                        override_source.name().as_str(),
                        override_source.parent_definition().subgraph().name().as_str(),
                        field.parent_definition().subgraph().name().as_str()));
            }
        }

        overrides.push(federated::Override {
            graph: federated::SubgraphId(field_subgraph.subgraph_id().idx()),
            label: override_directive
                .label
                .and_then(|label| ctx.subgraphs.walk(label).as_str().parse().ok())
                .unwrap_or_default(),
            from: ctx
                .subgraphs
                .iter_subgraphs()
                .position(|subgraph| subgraph.name().id == override_directive.from)
                .map(federated::SubgraphId)
                .map(federated::OverrideSource::Subgraph)
                .unwrap_or_else(|| federated::OverrideSource::Missing(ctx.insert_string(override_directive.from))),
        });
    }

    overrides
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
