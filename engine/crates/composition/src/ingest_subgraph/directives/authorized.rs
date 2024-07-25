use super::*;

pub(super) fn ingest(
    directive_site_id: DirectiveSiteId,
    directive: &ast::ConstDirective,
    subgraphs: &mut Subgraphs,
) -> Result<(), String> {
    let arguments = directive
        .get_argument("arguments")
        .and_then(|arg| match &arg.node {
            ConstValue::String(input) => Some(input),
            _ => None,
        })
        .map(|input| subgraphs.selection_set_from_str(input))
        .transpose()?;

    let fields = directive
        .get_argument("fields")
        .and_then(|arg| match &arg.node {
            ConstValue::String(requires) => Some(requires),
            _ => None,
        })
        .map(|fields| subgraphs.selection_set_from_str(fields))
        .transpose()?;

    let node = directive
        .get_argument("node")
        .and_then(|arg| match &arg.node {
            ConstValue::String(requires) => Some(requires),
            _ => None,
        })
        .map(|fields| subgraphs.selection_set_from_str(fields))
        .transpose()?;

    let metadata = directive
        .get_argument("metadata")
        .map(|value| ast_value_to_subgraph_value(&value.node, subgraphs));

    subgraphs.insert_authorized(
        directive_site_id,
        subgraphs::AuthorizedDirective {
            arguments,
            node,
            fields,
            metadata,
        },
    );

    Ok(())
}
