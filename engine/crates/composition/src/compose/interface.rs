use super::*;

pub(super) fn merge_interface_definitions<'a>(
    ctx: &mut Context<'a>,
    first: &DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
) {
    let composed_directives = collect_composed_directives(definitions.iter().map(|def| def.directives()), ctx);
    let interface_description = definitions.iter().find_map(|def| def.description());
    ctx.insert_interface(first.name(), interface_description, composed_directives);

    let mut all_fields: Vec<(StringId, _)> = definitions
        .iter()
        .flat_map(|def| def.fields().map(|field| (field.name().id, field)))
        .collect();

    all_fields.sort_by_key(|(name, _)| *name);

    let mut start = 0;

    while start < all_fields.len() {
        let (name, field) = all_fields[start];
        let end = start + all_fields[start..].partition_point(|(n, _)| *n == name);

        let description = field.description().map(|description| ctx.insert_string(description.id));

        let directive_containers = all_fields[start..end].iter().map(|(_, field)| field.directives());
        let composed_directives = collect_composed_directives(directive_containers, ctx);

        ctx.insert_field(ir::FieldIr {
            parent_name: first.name().id,
            field_name: field.name().id,
            field_type: field.r#type().id,
            arguments: Vec::new(),
            resolvable_in: None,
            provides: Vec::new(),
            requires: Vec::new(),
            overrides: Vec::new(),
            composed_directives,
            description,
        });

        start = end;
    }
}
