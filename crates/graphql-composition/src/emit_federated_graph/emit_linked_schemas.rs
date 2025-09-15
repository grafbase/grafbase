use super::*;

pub(crate) fn emit_linked_schemas(ctx: &mut Context<'_>) {
    let mut linked_schemas = std::mem::take(&mut ctx.composed_directive_linked_schemas);

    linked_schemas.sort();
    linked_schemas.dedup();

    let chunks = linked_schemas
        .chunk_by(|(linked_schema_id_a, _), (linked_schema_id_b, _)| linked_schema_id_a == linked_schema_id_b);

    for chunk in chunks {
        let (linked_schema_id, _) = &chunk[0];
        let url = ctx.insert_string(ctx.subgraphs.at(*linked_schema_id).url);

        let imports = chunk
            .iter()
            .map(|(_, imported_directive)| *imported_directive)
            .collect();

        ctx.out.linked_schemas.push(federated::LinkDirective { url, imports });
    }
}
