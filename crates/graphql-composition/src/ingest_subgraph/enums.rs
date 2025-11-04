use super::*;

pub(super) fn ingest_enum_values(
    ctx: &mut Context<'_>,
    parent_enum_id: DefinitionId,
    enum_type: ast::EnumDefinition<'_>,
) {
    for value in enum_type.values() {
        let name = ctx.subgraphs.strings.intern(value.value());
        let directives = ctx.subgraphs.new_directive_site();
        let description = value
            .description()
            .map(|description| ctx.subgraphs.strings.intern(description.to_cow()));

        ctx.subgraphs.push_enum_value(subgraphs::EnumValue {
            parent_enum_id,
            name,
            description,
            directives,
        });

        directives::ingest_directives(ctx, directives, value.directives(), |subgraphs| {
            subgraphs[subgraphs.at(parent_enum_id).name].as_ref().to_owned()
        });
    }
}
