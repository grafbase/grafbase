use super::schema_definitions::FederationDirectivesMatcher;
use crate::{subgraphs::DefinitionId, Subgraphs};
use async_graphql_parser::types as ast;
use async_graphql_value::ConstValue;

/// Returns whether the object is shareable.
pub(super) fn ingest_directives(
    definition_id: DefinitionId,
    type_definition: &ast::TypeDefinition,
    subgraphs: &mut Subgraphs,
    federation_directives_matcher: &FederationDirectivesMatcher<'_>,
) -> bool {
    let mut is_shareable = false;

    for directive in &type_definition.directives {
        if federation_directives_matcher.is_shareable(&directive.node.name.node) {
            is_shareable = true;
            continue;
        }

        if federation_directives_matcher.is_key(&directive.node.name.node) {
            let fields_arg = directive.node.get_argument("fields").map(|v| &v.node);
            let Some(ConstValue::String(fields_arg)) = fields_arg else {
                continue;
            };
            let Ok(selection_id) = subgraphs.selection_set_from_str(fields_arg) else {
                continue; // TODO: error handling in subgraph ingestion?
            };
            let is_resolvable = directive
                .node
                .get_argument("resolvable")
                .and_then(|v| match v.node {
                    ConstValue::Boolean(b) => Some(b),
                    _ => None,
                })
                .unwrap_or(true); // defaults to true
            subgraphs.push_key(definition_id, selection_id, is_resolvable);
        }
    }

    is_shareable
}
