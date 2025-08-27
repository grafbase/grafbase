use super::*;

pub(super) fn ingest_enum_values(
    ctx: &mut Context<'_>,
    definition_id: DefinitionId,
    enum_type: ast::EnumDefinition<'_>,
) {
    for value in enum_type.values() {
        let value_name = ctx.subgraphs.strings.intern(value.value());
        let value_directives = ctx.subgraphs.new_directive_site();

        ctx.subgraphs
            .push_enum_value(definition_id, value_name, value_directives);

        directives::ingest_directives(ctx, value_directives, value.directives(), |subgraphs| {
            subgraphs[subgraphs.at(definition_id).name].as_ref().to_owned()
        });
    }
}
