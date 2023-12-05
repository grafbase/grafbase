use super::*;

/// The arguments of a federated graph's fields are the interseciton of the subgraph's arguments for
/// that field. Returns (arg_name, arg_type, is_inaccessible).
pub(super) fn merge_field_arguments<'a>(
    first: FieldWalker<'a>,
    fields: &[FieldWalker<'a>],
    ctx: &mut Context<'a>,
) -> Vec<ir::ArgumentIr> {
    let mut intersection: Vec<StringId> = first.arguments().map(|arg| arg.name().id).collect();
    let mut buf = HashSet::new();

    for field in &fields[1..] {
        buf.clear();
        buf.extend(field.arguments().map(|arg| arg.name().id));
        intersection.retain(|value| buf.contains(value));
    }

    for argument in first.arguments().filter(|arg| {
        !arg.directives().inaccessible()
            && arg
                .r#type()
                .definition(first.parent_definition().subgraph().id)
                .map(|def| def.directives().inaccessible())
                .unwrap_or(false)
    }) {
        ctx.diagnostics.push_fatal(format!(
            "The argument `{}` on `{}` is of an @inaccessible type, but is itself not marked as @inaccessible.",
            argument.name().as_str(),
            first.name().as_str(),
        ));
    }

    intersection
        .into_iter()
        .map(|argument_name| {
            let argument_type = first.argument_by_name(argument_name).unwrap().r#type().id;
            let directive_containers = fields
                .iter()
                .filter_map(|field| field.argument_by_name(argument_name))
                .map(|arg| arg.directives());

            let composed_directives = collect_composed_directives(directive_containers, ctx);

            ir::ArgumentIr {
                argument_name,
                argument_type,
                composed_directives,
            }
        })
        .collect()
}

pub(super) fn compose_object_fields<'a>(
    object_is_shareable: bool,
    first: FieldWalker<'a>,
    fields: &[FieldWalker<'a>],
    ctx: &mut Context<'a>,
) {
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

    let first_is_part_of_key = first.is_part_of_key();
    if fields
        .iter()
        .any(|field| field.is_part_of_key() != first_is_part_of_key)
    {
        let name = format!(
            "{}.{}",
            first.parent_definition().name().as_str(),
            first.name().as_str()
        );
        let (key_subgraphs, non_key_subgraphs) = fields
            .iter()
            .partition::<Vec<FieldWalker<'_>>, _>(|field| field.is_part_of_key());

        ctx.diagnostics.push_fatal(format!(
            "The field `{name}` is part of `@key` in {} but not in {}",
            key_subgraphs
                .into_iter()
                .map(|f| f.parent_definition().subgraph().name().as_str())
                .join(", "),
            non_key_subgraphs
                .into_iter()
                .map(|f| f.parent_definition().subgraph().name().as_str())
                .join(", "),
        ));
    }

    if fields.iter().any(|field| {
        !field.directives().inaccessible()
            && field
                .r#type()
                .definition(field.parent_definition().subgraph().id)
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

    let resolvable_in = fields
        .first()
        .filter(|_| fields.len() == 1)
        .map(|field| federated::SubgraphId(field.parent_definition().subgraph().id.idx()));

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

    let overrides = collect_overrides(fields, ctx);
    let description = fields
        .iter()
        .find_map(|f| f.description().map(|d| ctx.insert_string(d.id)));

    let composed_directives = collect_composed_directives(fields.iter().map(|f| f.directives()), ctx);

    ctx.insert_field(ir::FieldIr {
        parent_name: first.parent_definition().name().id,
        field_name: first.name().id,
        field_type: first.r#type().id,
        arguments,
        resolvable_in,
        provides,
        requires,
        composed_directives,
        overrides,
        description,
    });
}

fn collect_overrides(fields: &[FieldWalker<'_>], ctx: &mut Context<'_>) -> Vec<federated::Override> {
    let mut overrides = Vec::new();

    for (field, from) in fields.iter().filter_map(|f| Some(f).zip(f.directives().r#override())) {
        let field_subgraph = field.parent_definition().subgraph();

        if from.id == field_subgraph.name().id {
            ctx.diagnostics.push_fatal(format!(
                r#"Source and destination subgraphs "{}" are the same for overridden field "{}.{}""#,
                from.as_str(),
                field.parent_definition().name().as_str(),
                field.name().as_str()
            ));
            continue;
        }

        if let Some(override_source) = fields
            .iter()
            .find(|f| f.parent_definition().subgraph().name().id == from.id)
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
            graph: federated::SubgraphId(field_subgraph.id.idx()),
            from: ctx
                .subgraphs
                .iter_subgraphs()
                .position(|subgraph| subgraph.name().id == from.id)
                .map(federated::SubgraphId)
                .map(federated::OverrideSource::Subgraph)
                .unwrap_or_else(|| federated::OverrideSource::Missing(ctx.insert_string(from.id))),
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
