use super::schema_definitions::FederationDirectivesMatcher;
use crate::{
    subgraphs::{DefinitionId, StringId},
    Subgraphs,
};
use async_graphql_parser::{
    types::{self as ast},
    Positioned,
};
use async_graphql_value::ConstValue;

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

        let is_inaccessible = field
            .directives
            .iter()
            .any(|directive| federation_directives_matcher.is_inaccessible(directive.node.name.node.as_str()));

        let provides = field
            .directives
            .iter()
            .find(|directive| federation_directives_matcher.is_provides(directive.node.name.node.as_str()))
            .and_then(|directive| directive.node.get_argument("fields"))
            .and_then(|v| match &v.node {
                ConstValue::String(s) => Some(s.as_str()),
                _ => None,
            });

        let requires = field
            .directives
            .iter()
            .find(|directive| federation_directives_matcher.is_requires(directive.node.name.node.as_str()))
            .and_then(|directive| directive.node.get_argument("fields"))
            .and_then(|v| match &v.node {
                ConstValue::String(s) => Some(s.as_str()),
                _ => None,
            });

        let deprecated = super::find_deprecated_directive(&field.directives, subgraphs);
        let overrides = find_override_directive(&field.directives, subgraphs, federation_directives_matcher);
        let tags = super::find_tag_directives(&field.directives);
        let field_type = subgraphs.intern_field_type(&field.ty.node);
        let field_id = subgraphs
            .push_field(crate::subgraphs::FieldIngest {
                parent_definition_id: definition_id,
                field_name: &field.name.node,
                field_type,
                is_shareable,
                is_external,
                is_inaccessible,
                provides,
                requires,
                deprecated,
                tags,
                overrides,
            })
            .unwrap();

        super::field::ingest_field_arguments(field_id, &field.arguments, subgraphs);
    }
}

fn find_override_directive(
    directives: &[Positioned<ast::ConstDirective>],
    subgraphs: &mut Subgraphs,
    federation_directives_matcher: &FederationDirectivesMatcher<'_>,
) -> Option<StringId> {
    directives
        .iter()
        .find(|directive| federation_directives_matcher.is_override(directive.node.name.node.as_str()))
        .and_then(|directive| directive.node.get_argument("from"))
        .and_then(|v| match &v.node {
            ConstValue::String(s) => Some(subgraphs.strings.intern(s.as_str())),
            _ => None,
        })
}
