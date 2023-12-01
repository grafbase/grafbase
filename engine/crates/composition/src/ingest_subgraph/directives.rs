use super::*;

pub(super) fn ingest_directives(
    directives: DirectiveContainerId,
    directives_node: &[Positioned<ast::ConstDirective>],
    subgraphs: &mut Subgraphs,
    federation_directives_matcher: &FederationDirectivesMatcher<'_>,
) {
    for directive in directives_node {
        let directive_name = &directive.node.name.node;
        if federation_directives_matcher.is_shareable(directive_name) {
            subgraphs.set_shareable(directives);
            continue;
        }

        if federation_directives_matcher.is_external(directive_name) {
            subgraphs.set_external(directives);
            continue;
        }

        if federation_directives_matcher.is_interface_object(directive_name) {
            subgraphs.set_interface_object(directives);
            continue;
        }

        if federation_directives_matcher.is_inaccessible(directive_name) {
            subgraphs.set_inaccessible(directives);
            continue;
        }

        if federation_directives_matcher.is_override(directive_name) {
            let from = directive
                .node
                .get_argument("from")
                .and_then(|v| match &v.node {
                    ConstValue::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .map(|s| subgraphs.strings.intern(s));

            let Some(from) = from else { continue };

            subgraphs.set_override(directives, from);
            continue;
        }

        if federation_directives_matcher.is_requires(directive_name) {
            let fields_arg = directive.node.get_argument("fields").map(|v| &v.node);
            let Some(ConstValue::String(fields_arg)) = fields_arg else {
                continue;
            };
            subgraphs.insert_requires(directives, fields_arg).ok();
            continue;
        }

        if federation_directives_matcher.is_provides(directive_name) {
            let fields_arg = directive.node.get_argument("fields").map(|v| &v.node);
            let Some(ConstValue::String(fields_arg)) = fields_arg else {
                continue;
            };
            subgraphs.insert_provides(directives, fields_arg).ok();
            continue;
        }

        if directive_name == "tag" {
            let Some(value) = directive.node.get_argument("name") else {
                continue;
            };

            if let async_graphql_value::ConstValue::String(s) = &value.node {
                subgraphs.insert_tag(directives, s.as_str());
            }
        }

        if directive_name == "deprecated" {
            let reason = directive.node.get_argument("reason").and_then(|v| match &v.node {
                async_graphql_value::ConstValue::String(s) => Some(s.as_str()),
                _ => None,
            });

            subgraphs.insert_deprecated(directives, reason);
        }
    }
}

pub(super) fn ingest_keys(
    definition_id: DefinitionId,
    directives_node: &[Positioned<ast::ConstDirective>],
    subgraphs: &mut Subgraphs,
    federation_directives_matcher: &FederationDirectivesMatcher<'_>,
) {
    for directive in directives_node {
        let directive_name = &directive.node.name.node;

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
