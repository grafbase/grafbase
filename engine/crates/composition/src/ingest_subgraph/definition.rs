use super::*;

pub(super) fn ingest_directives(
    definition_id: DefinitionId,
    type_definition: &ast::TypeDefinition,
    subgraphs: &mut Subgraphs,
    federation_directives_matcher: &FederationDirectivesMatcher<'_>,
) {
    for directive in &type_definition.directives {
        let directive_name = &directive.node.name.node;
        if federation_directives_matcher.is_shareable(directive_name) {
            subgraphs.set_shareable(definition_id);
            continue;
        }

        if federation_directives_matcher.is_external(directive_name) {
            subgraphs.set_external(definition_id);
            continue;
        }

        if federation_directives_matcher.is_interface_object(directive_name) {
            subgraphs.set_interface_object(definition_id);
            continue;
        }

        if federation_directives_matcher.is_inaccessible(directive_name) {
            subgraphs.set_inaccessible(definition_id);
            continue;
        }

        if federation_directives_matcher.is_key(directive_name) {
            let fields_arg = directive.node.get_argument("fields").map(|v| &v.node);
            let Some(ConstValue::String(fields_arg)) = fields_arg else {
                continue;
            };
            let is_resolvable = directive
                .node
                .get_argument("resolvable")
                .and_then(|v| match v.node {
                    ConstValue::Boolean(b) => Some(b),
                    _ => None,
                })
                .unwrap_or(true); // defaults to true
            subgraphs.push_key(definition_id, fields_arg, is_resolvable).ok();
        }
    }
}
