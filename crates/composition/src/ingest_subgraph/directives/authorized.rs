use super::*;

pub(super) fn ingest(
    directive_site_id: DirectiveSiteId,
    directive: ast::Directive<'_>,
    subgraphs: &mut Subgraphs,
) -> Result<(), String> {
    let arguments = directive
        .argument("arguments")
        .and_then(|arg| arg.value().as_str())
        .map(|input| subgraphs.selection_set_from_str(input, "authorized", "arguments"))
        .transpose()?;

    let fields = directive
        .argument("fields")
        .and_then(|arg| arg.value().as_str())
        .map(|fields| subgraphs.selection_set_from_str(fields, "authorized", "fields"))
        .transpose()?;

    let node = directive
        .argument("node")
        .and_then(|arg| arg.value().as_str())
        .map(|fields| subgraphs.selection_set_from_str(fields, "authorized", "node"))
        .transpose()?;

    let metadata = directive
        .argument("metadata")
        .map(|arg| ast_value_to_subgraph_value(arg.value(), subgraphs));

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
