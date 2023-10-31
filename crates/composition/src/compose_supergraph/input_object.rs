use crate::strings::StringId;

use super::*;
use std::collections::{HashMap, HashSet};

pub(super) fn merge_input_object_definitions(
    ctx: &mut Context<'_>,
    first: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    // We want to take the intersection of the field sets.
    let mut common_fields: HashMap<StringId, _> =
        first.fields().map(|field| (field.name(), field)).collect();
    let mut fields_buf = HashSet::<StringId>::new();

    for input_object in definitions {
        fields_buf.clear();
        fields_buf.extend(input_object.fields().map(|f| f.name()));
        common_fields.retain(|field_name, _| fields_buf.contains(&field_name));
    }

    // Check that no required field was excluded.
    for field in definitions
        .iter()
        .flat_map(|input_object| input_object.fields())
    {
        if field.r#type().is_required() && !common_fields.contains_key(&field.name()) {
            ctx.diagnostics.push_fatal(format!(
                "The {input_type_name}.{field_name} field is not defined in all subgraphs, but it is required in {bad_subgraph}",
                input_type_name = first.name_str(),
                field_name = field.name_str(),
                bad_subgraph = field.parent_definition().subgraph().name_str(),
            ));
        }
    }

    ctx.supergraph
        .insert_definition(first.name(), DefinitionKind::InputObject);

    for (_, field) in common_fields {
        ctx.supergraph.insert_field(
            first.name(),
            field.name(),
            field.r#type().type_name(),
            Default::default(),
        );
    }
}
