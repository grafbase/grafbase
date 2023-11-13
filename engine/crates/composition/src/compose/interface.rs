use super::*;
use crate::subgraphs::{FieldId, StringId};

pub(super) fn merge_interface_definitions(
    ctx: &mut Context<'_>,
    first: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    let mut all_fields: indexmap::IndexMap<StringId, FieldId> = indexmap::IndexMap::new();

    for field in definitions.iter().flat_map(|def| def.fields()) {
        all_fields.entry(field.name().id).or_insert(field.id);
    }

    ctx.insert_interface(first.name());

    for field in all_fields.values() {
        let field = first.walk(*field);
        ctx.insert_field(first.name().id, field.name().id, field.r#type().id, Vec::new(), None);
    }
}
