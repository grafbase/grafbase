use super::*;

pub(super) fn merge_input_object_definitions(
    ctx: &mut Context<'_>,
    first: &DefinitionView<'_>,
    definitions: &[DefinitionView<'_>],
) {
    let mut fields_range: Option<federated::InputValueDefinitions> = None;
    let description = definitions
        .iter()
        .find_map(|def| def.description)
        .map(|d| ctx.subgraphs[d].as_ref());

    let input_object_name = ctx.insert_string(first.name);

    // We want to take the intersection of the field sets.
    let intersection: HashSet<StringId> = first
        .id
        .fields(ctx.subgraphs)
        .map(|field| field.name)
        .filter(|field_name| {
            definitions[1..]
                .iter()
                .all(|def| def.id.field_by_name(ctx.subgraphs, *field_name).is_some())
        })
        .collect();

    let mut all_fields: Vec<_> = definitions
        .iter()
        .flat_map(|def| def.id.fields(ctx.subgraphs))
        .collect();

    all_fields.sort_by_key(|field| field.name);

    let mut start = 0;

    while start < all_fields.len() {
        let field_name = all_fields[start].name;
        let end = all_fields[start..].partition_point(|field| field.name == field_name) + start;
        let fields = &all_fields[start..end];

        start = end;

        // Check that no required field was excluded.
        if !intersection.contains(&field_name) {
            if let Some(required_field) = fields.iter().find(|field| field.r#type.is_required()) {
                ctx.diagnostics.push_fatal(format!(
                    "The {input_type_name}.{field_name} field is not defined in all subgraphs, but it is required in {bad_subgraph}",
                    input_type_name = ctx.subgraphs[first.name],
                    field_name = ctx.subgraphs[required_field.name],
                    bad_subgraph = ctx.subgraphs[ctx.subgraphs.at(ctx.subgraphs.at(required_field.parent_definition_id).subgraph_id).name],
                ));
            }
            continue;
        }

        let directive_containers = fields.iter().map(|field| field.directives);
        let mut directives = collect_composed_directives(directive_containers, ctx);

        let description = fields
            .iter()
            .find_map(|field| field.description)
            .map(|description| ctx.insert_string(description));

        let Some(composed_field_type) = fields::compose_input_field_types(ctx, fields.iter().copied()) else {
            continue;
        };

        directives.extend(fields.iter().map(|field| {
            ir::Directive::JoinInputField(ir::JoinInputFieldDirective {
                subgraph_id: ctx.subgraphs.at(field.parent_definition_id).subgraph_id.idx().into(),
                r#type: if field.r#type != composed_field_type {
                    Some(field.r#type)
                } else {
                    None
                },
            })
        }));

        let default = fields
            .iter()
            .find_map(|field| field.input_field_default_value.as_ref())
            .cloned();

        let name = ctx.insert_string(field_name);
        let id = ctx.insert_input_value_definition(ir::InputValueDefinitionIr {
            name,
            r#type: composed_field_type,
            directives,
            description,
            default,
        });

        if let Some((_start, len)) = &mut fields_range {
            *len += 1;
        } else {
            fields_range = Some((id, 1));
        }
    }

    let mut directives = collect_composed_directives(definitions.iter().map(|def| def.directives), ctx);
    directives.extend(create_join_type_from_definitions(definitions));
    let fields = fields_range.unwrap_or(federated::NO_INPUT_VALUE_DEFINITION);
    ctx.insert_input_object(input_object_name, description, directives, fields);
}
