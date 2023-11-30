use super::*;

pub(crate) fn merge_scalar_definitions(
    first: DefinitionWalker<'_>,
    definitions: &[DefinitionWalker<'_>],
    ctx: &mut Context<'_>,
) {
    let is_inaccessible = definitions.iter().any(|definition| definition.is_inaccessible());

    ctx.insert_scalar(first.name(), is_inaccessible);
}
