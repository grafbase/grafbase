use super::*;

pub(crate) fn merge_scalar_definitions<'a>(
    first: DefinitionWalker<'a>,
    definitions: &[DefinitionWalker<'a>],
    ctx: &mut Context<'a>,
) {
    if first.name().as_str() == "join__FieldSet" {
        return;
    }
    let directive_containers = definitions.iter().map(|def| def.view().directives);
    let directives = collect_composed_directives(directive_containers, ctx);
    let description = definitions
        .iter()
        .find_map(|def| def.view().description)
        .map(|d| ctx.subgraphs[d].as_ref());

    ctx.insert_scalar(first.name().as_str(), description, directives);
}
