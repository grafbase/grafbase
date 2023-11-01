use super::*;
use crate::{strings::StringId, subgraphs::FieldId};

pub(super) fn merge_interface_definitions(
    ctx: &mut Context<'_>,
    first: &DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
) {
    let mut all_fields: indexmap::IndexMap<StringId, FieldId> = indexmap::IndexMap::new();

    for field in definitions.iter().flat_map(|def| def.fields()) {
        all_fields.entry(field.name()).or_insert(field.id);
    }

    ctx.supergraph
        .insert_definition(first.name(), DefinitionKind::Interface);

    for field in all_fields.values() {
        let field = first.walk(*field);
        ctx.supergraph.insert_field(
            first.name(),
            field.name(),
            field.r#type().type_name(),
            Vec::new(),
        );
    }
}
