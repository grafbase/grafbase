use super::*;
use crate::composition_ir as ir;
use std::collections::{HashMap, HashSet};

pub(super) fn merge_input_object_definitions(
    ctx: &mut Context<'_>,
    first: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    let description = definitions.iter().find_map(|def| def.description());

    // We want to take the intersection of the field sets.
    let mut common_fields: HashMap<StringId, _> = first.fields().map(|field| (field.name().id, field)).collect();
    let mut fields_buf = HashSet::<StringId>::new();

    let is_inaccessible = definitions.iter().any(|definition| definition.is_inaccessible());
    let inaccessible_fields: HashSet<_> = definitions
        .iter()
        .flat_map(|def| def.fields())
        .filter(|f| f.is_inaccessible())
        .map(|field| field.name().id)
        .collect();

    for input_object in definitions {
        fields_buf.clear();
        fields_buf.extend(input_object.fields().map(|f| f.name().id));
        common_fields.retain(|field_name, _| fields_buf.contains(field_name));
    }

    // Check that no required field was excluded.
    for field in definitions.iter().flat_map(|input_object| input_object.fields()) {
        if field.r#type().is_required() && !common_fields.contains_key(&field.name().id) {
            ctx.diagnostics.push_fatal(format!(
                "The {input_type_name}.{field_name} field is not defined in all subgraphs, but it is required in {bad_subgraph}",
                input_type_name = first.name().as_str(),
                field_name = field.name().as_str(),
                bad_subgraph = field.parent_definition().subgraph().name().as_str(),
            ));
        }
    }

    ctx.insert_input_object(first.name(), is_inaccessible, description);

    for field in first.fields().filter(|f| common_fields.contains_key(&f.name().id)) {
        let composed_directives = if inaccessible_fields.contains(&field.name().id) {
            vec![federated::Directive {
                name: ctx.insert_static_str("inaccessible"),
                arguments: Vec::new(),
            }]
        } else {
            Vec::new()
        };

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
