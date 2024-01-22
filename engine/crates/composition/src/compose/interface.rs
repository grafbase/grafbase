use super::*;

pub(super) fn merge_interface_definitions<'a>(
    ctx: &mut Context<'a>,
    first: &DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
) {
    let composed_directives = collect_composed_directives(definitions.iter().map(|def| def.directives()), ctx);
    let interface_description = definitions.iter().find_map(|def| def.description()).map(|d| d.as_str());
    let interface_name = ctx.insert_string(first.name().id);
    let interface_id = ctx.insert_interface(interface_name, interface_description, composed_directives);

    let mut all_fields: Vec<(StringId, _)> = definitions
        .iter()
        .flat_map(|def| def.fields().map(|field| (field.name().id, field)))
        .collect();

    all_fields.sort_by_key(|(name, _)| *name);

    let mut start = 0;

    while start < all_fields.len() {
        let (name, field) = all_fields[start];
        let end = start + all_fields[start..].partition_point(|(n, _)| *n == name);
        let fields = &all_fields[start..end];

        start = end;

        let description = field.description().map(|description| ctx.insert_string(description.id));

        let directive_containers = fields.iter().map(|(_, field)| field.directives());
        let composed_directives = collect_composed_directives(directive_containers, ctx);

        let Some(field_type) = fields::compose_output_field_types(fields.iter().map(|(_, field)| *field), ctx) else {
            continue;
        };

        ctx.insert_field(ir::FieldIr {
            parent_definition: federated::Definition::Interface(interface_id),
            field_name: field.name().id,
            field_type,
            arguments: Vec::new(),
            resolvable_in: Vec::new(),
            provides: Vec::new(),
            requires: Vec::new(),
            overrides: Vec::new(),
            composed_directives,
            description,
        });
    }

    all_fields.dedup_by_key(|(name, _)| *name);

    check_implementers(first.name().id, all_fields.iter().map(|(name, _)| *name), ctx);
}

fn check_implementers(
    interface_name: StringId,
    field_names: impl Iterator<Item = StringId> + Clone,
    ctx: &mut Context<'_>,
) {
    for implementer_name in ctx.subgraphs.iter_implementers_for_interface(interface_name) {
        let field_names = field_names.clone();

        for field_name in field_names {
            if !ctx
                .subgraphs
                .iter_definitions_with_name(implementer_name)
                .any(|(_, def)| ctx.subgraphs.walk(def).find_field(field_name).is_some())
            {
                ctx.diagnostics.push_fatal(format!(
                    "The `{}.{}` field is not implemented by `{}`, but it should be.",
                    ctx.subgraphs.walk(interface_name).as_str(),
                    ctx.subgraphs.walk(field_name).as_str(),
                    ctx.subgraphs.walk(implementer_name).as_str(),
                ));
            }
        }
    }
}
