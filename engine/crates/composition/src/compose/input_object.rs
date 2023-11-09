use std::collections::{HashMap, HashSet};

use super::*;

pub(super) fn merge_input_object_definitions(
    ctx: &mut Context<'_>,
    first: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    // We want to take the intersection of the field sets.
    let mut common_fields: HashMap<StringId, _> = first.fields().map(|field| (field.name().id, field)).collect();
    let mut fields_buf = HashSet::<StringId>::new();

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

    ctx.insert_input_object(first.name());

    for field in first.fields().filter(|f| common_fields.contains_key(&f.name().id)) {
        ctx.insert_field(
            first.name().id,
            field.name().id,
            field.r#type().id,
            Default::default(),
            Default::default(),
        );
    }
}
