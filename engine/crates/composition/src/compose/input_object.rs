use super::*;

pub(super) fn merge_input_object_definitions(
    ctx: &mut Context<'_>,
    first: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    let description = definitions.iter().find_map(|def| def.description());

    // We want to take the intersection of the field sets.
    let mut common_fields: Vec<StringId> = first.fields().map(|field| field.name().id).collect();
    let mut fields_buf = HashSet::<StringId>::new();

    for input_object in definitions {
        fields_buf.clear();
        fields_buf.extend(input_object.fields().map(|f| f.name().id));
        common_fields.retain(|field_name| fields_buf.contains(field_name));
    }

    // Check that no required field was excluded.
    for field in definitions.iter().flat_map(|input_object| input_object.fields()) {
        if field.r#type().is_required() && !common_fields.contains(&field.name().id) {
            ctx.diagnostics.push_fatal(format!(
                "The {input_type_name}.{field_name} field is not defined in all subgraphs, but it is required in {bad_subgraph}",
                input_type_name = first.name().as_str(),
                field_name = field.name().as_str(),
                bad_subgraph = field.parent_definition().subgraph().name().as_str(),
            ));
        }
    }

    let composed_directives = collect_composed_directives(definitions.iter().map(|def| def.directives()), ctx);

    ctx.insert_input_object(first.name(), description, composed_directives);

    for field_name in common_fields {
        let field = first.find_field(field_name).unwrap(); // safe because we just filtered by common_fields
        let directive_containers = definitions
            .iter()
            .filter_map(|input_object| input_object.find_field(field_name))
            .map(|field| field.directives());

        let composed_directives = collect_composed_directives(directive_containers, ctx);

        let field_name = field.name();

        let description = definitions
            .iter()
            .filter_map(|definition| definition.find_field(field_name.id))
            .find_map(|field| field.description())
            .map(|description| ctx.insert_string(description.id));

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
    }
}
