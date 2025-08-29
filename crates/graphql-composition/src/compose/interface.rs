use super::*;

pub(super) fn merge_interface_definitions<'a>(
    ctx: &mut Context<'a>,
    first: &DefinitionView<'a>,
    definitions: &[DefinitionView<'a>],
) {
    let mut directives = collect_composed_directives(definitions.iter().map(|def| def.directives), ctx);
    directives.extend(create_join_type_from_definitions(definitions));
    let interface_description = definitions
        .iter()
        .find_map(|def| def.description)
        .map(|d| ctx.subgraphs[d].as_ref());
    let interface_name = ctx.insert_string(first.name);
    ctx.insert_interface(interface_name, interface_description, directives);

    fields::for_each_field_group(ctx.subgraphs, definitions, |fields| {
        if fields.iter().any(|field| field.directives.shareable(ctx.subgraphs)) {
            ctx.diagnostics.push_fatal(format!(
                "The field {}.{} is marked as shareable but this is not allowed on interfaces.",
                ctx.subgraphs[first.name].as_ref(),
                ctx.subgraphs[fields.first().unwrap().name].as_ref()
            ));
        }
    });

    let fields = object::compose_fields(ctx, definitions, interface_name);
    let field_names = fields.iter().map(|field| field.field_name).collect::<Vec<_>>();
    for field in fields {
        ctx.insert_field(field);
    }

    check_implementers(first.name, &field_names, ctx);
}

fn check_implementers(interface_name: StringId, field_names: &[subgraphs::StringId], ctx: &mut Context<'_>) {
    for implementer_name in ctx.subgraphs.iter_implementers_for_interface(interface_name) {
        for field_name in field_names {
            if !ctx
                .subgraphs
                .iter_definitions_with_name(implementer_name)
                .any(|(_, def)| def.field_by_name(ctx.subgraphs, *field_name).is_some())
            {
                ctx.diagnostics.push_fatal(format!(
                    "The `{}.{}` field is not implemented by `{}`, but it should be.",
                    ctx.subgraphs[interface_name], ctx.subgraphs[*field_name], ctx.subgraphs[implementer_name],
                ));
            }
        }
    }
}
