use super::*;
use crate::{
    composition_ir as ir,
    subgraphs::{FieldId, StringId},
};

pub(super) fn merge_interface_definitions(
    ctx: &mut Context<'_>,
    first: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    let mut all_fields: indexmap::IndexMap<StringId, FieldId> = indexmap::IndexMap::new();
    let is_inaccessible = definitions.iter().any(|definition| definition.is_inaccessible());
    let interface_description = definitions.iter().find_map(|def| def.description());

    for field in definitions.iter().flat_map(|def| def.fields()) {
        all_fields.entry(field.name().id).or_insert(field.id);
    }

    ctx.insert_interface(first.name(), is_inaccessible, interface_description);

    for field in all_fields.values() {
        let field = first.walk(*field);
        let description = field.description().map(|description| ctx.insert_string(description.id));
        ctx.insert_field(ir::FieldIr {
            parent_name: first.name().id,
            field_name: field.name().id,
            field_type: field.r#type().id,
            arguments: Vec::new(),
            resolvable_in: None,
            provides: Vec::new(),
            requires: Vec::new(),
            composed_directives: Vec::new(),
            overrides: Vec::new(),
            description,
        });
    }
}
