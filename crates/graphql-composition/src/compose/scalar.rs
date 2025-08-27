use super::*;

pub(crate) fn merge_scalar_definitions<'a>(
    first: DefinitionView<'_>,
    definitions: &[DefinitionView<'_>],
    ctx: &mut Context<'a>,
) {
    if ctx.subgraphs[first.name].as_ref() == "join__FieldSet" {
        return;
    }
    let directive_containers = definitions.iter().map(|def| def.directives);
    let directives = collect_composed_directives(directive_containers, ctx);
    let description = definitions
        .iter()
        .find_map(|def| def.description)
        .map(|d| ctx.subgraphs[d].as_ref());

    ctx.insert_scalar(ctx.subgraphs[first.name].as_ref(), description, directives);
}
