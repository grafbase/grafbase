use async_graphql_parser::types as ast;
use async_graphql_value::ConstValue;

use super::schema_definitions::FederationDirectivesMatcher;
use crate::{subgraphs::DefinitionId, Subgraphs};

/// Returns whether the object is shareable.
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

pub(super) fn ingest_fields(
    definition_id: DefinitionId,
    object_type: &ast::ObjectType,
    federation_directives_matcher: &FederationDirectivesMatcher<'_>,
    subgraphs: &mut Subgraphs,
) {
    let object = subgraphs.walk(definition_id);
    let object_is_shareable = object.is_shareable();
    let object_is_external = object.is_external();

    for field in &object_type.fields {
        let field = &field.node;
        let is_shareable = object_is_shareable
            || field
                .directives
                .iter()
                .any(|directive| federation_directives_matcher.is_shareable(directive.node.name.node.as_str()));

        let is_external = object_is_external
            || field
                .directives
                .iter()
                .any(|directive| federation_directives_matcher.is_external(directive.node.name.node.as_str()));

        let provides = field
            .directives
            .iter()
            .find(|directive| federation_directives_matcher.is_provides(directive.node.name.node.as_str()))
            .and_then(|directive| directive.node.get_argument("fields"))
            .and_then(|v| match &v.node {
                ConstValue::String(s) => Some(s.as_str()),
                _ => None,
            });

        let type_id = subgraphs.intern_field_type(&field.ty.node);
        let field_id = subgraphs
            .push_field(
                definition_id,
                &field.name.node,
                type_id,
                is_shareable,
                is_external,
                provides,
            )
            .unwrap();

        super::field::ingest_field_arguments(field_id, &field.arguments, subgraphs);
    }
}
