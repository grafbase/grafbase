use super::*;

pub(crate) fn merge_scalar_definitions<'a>(
    first: DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
    ctx: &mut Context<'a>,
) {
    let directive_containers = definitions.iter().map(|def| def.directives());
    let directives = collect_composed_directives(directive_containers, ctx);
    let description = definitions.iter().find_map(|def| def.description()).map(|d| d.as_str());

    ctx.insert_scalar(first.name().as_str(), description, directives);
}
