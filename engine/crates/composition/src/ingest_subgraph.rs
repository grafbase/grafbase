//! This is a separate module because we want to use only the public API of [Subgraphs] and avoid
//! mixing GraphQL parser logic and types with our internals.

mod directives;
mod enums;
mod fields;
mod nested_key_fields;
mod schema_definitions;

use self::{nested_key_fields::ingest_nested_key_fields, schema_definitions::*};
use crate::{
    subgraphs::{self, DefinitionId, DefinitionKind, DirectiveContainerId, SubgraphId},
    Subgraphs,
};
use async_graphql_parser::{types as ast, Positioned};
use async_graphql_value::ConstValue;

pub(crate) fn ingest_subgraph(document: &ast::ServiceDocument, name: &str, url: &str, subgraphs: &mut Subgraphs) {
    let subgraph_id = subgraphs.push_subgraph(name, url);

    let federation_directives_matcher = ingest_schema_definitions(document);

    ingest_top_level_definitions(subgraph_id, document, subgraphs, &federation_directives_matcher);
    ingest_definition_bodies(subgraph_id, document, subgraphs, &federation_directives_matcher);
    ingest_nested_key_fields(subgraph_id, subgraphs);
}

fn ingest_top_level_definitions(
    subgraph_id: SubgraphId,
    document: &ast::ServiceDocument,
    subgraphs: &mut Subgraphs,
    federation_directives_matcher: &FederationDirectivesMatcher<'_>,
) {
    for definition in &document.definitions {
        match definition {
            ast::TypeSystemDefinition::Type(type_definition) => {
                let type_name = &type_definition.node.name.node;
                let description = type_definition
                    .node
                    .description
                    .as_ref()
                    .map(|description| subgraphs.strings.intern(description.node.as_str()));

                let directives = subgraphs.new_directive_container();

                let definition_id = match &type_definition.node.kind {
                    ast::TypeKind::Object(_) => subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::Object,
                        description,
                        directives,
                    ),
                    ast::TypeKind::Interface(_interface_type) => subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::Interface,
                        description,
                        directives,
                    ),
                    ast::TypeKind::Union(_) => subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::Union,
                        description,
                        directives,
                    ),
                    ast::TypeKind::InputObject(_) => subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::InputObject,
                        description,
                        directives,
                    ),

                    ast::TypeKind::Scalar => subgraphs.push_definition(
                        subgraph_id,
                        type_name,
                        DefinitionKind::Scalar,
                        description,
                        directives,
                    ),

                    ast::TypeKind::Enum(enum_type) => {
                        let definition_id = subgraphs.push_definition(
                            subgraph_id,
                            type_name,
                            DefinitionKind::Enum,
                            description,
                            directives,
                        );
                        enums::ingest_enum(definition_id, enum_type, subgraphs, federation_directives_matcher);
                        definition_id
                    }
                };

                directives::ingest_directives(
                    directives,
                    &type_definition.node.directives,
                    subgraphs,
                    federation_directives_matcher,
                );
                directives::ingest_keys(
                    definition_id,
                    &type_definition.node.directives,
                    subgraphs,
                    federation_directives_matcher,
                );
            }
            ast::TypeSystemDefinition::Schema(_) | ast::TypeSystemDefinition::Directive(_) => (),
        }
    }
}

fn ingest_definition_bodies(
    subgraph_id: SubgraphId,
    document: &ast::ServiceDocument,
    subgraphs: &mut Subgraphs,
    federation_directives_matcher: &FederationDirectivesMatcher<'_>,
) {
    let type_definitions = document.definitions.iter().filter_map(|def| match def {
        ast::TypeSystemDefinition::Type(ty) => Some(ty),
        _ => None,
    });

    for definition in type_definitions {
        match &definition.node.kind {
            ast::TypeKind::Union(union) => {
                let union_id = subgraphs.definition_by_name(&definition.node.name.node, subgraph_id);

                for member in &union.members {
                    let member_id = subgraphs.definition_by_name(&member.node, subgraph_id);
                    subgraphs.push_union_member(union_id, member_id);
                }
            }
            ast::TypeKind::InputObject(input_object) => {
                let definition_id = subgraphs.definition_by_name(&definition.node.name.node, subgraph_id);
                fields::ingest_input_fields(
                    definition_id,
                    &input_object.fields,
                    federation_directives_matcher,
                    subgraphs,
                );
            }
            ast::TypeKind::Interface(interface) => {
                let definition_id = subgraphs.definition_by_name(&definition.node.name.node, subgraph_id);
                let definition_name = subgraphs.walk(definition_id).name().id;

                for implemented_interface in &interface.implements {
                    let implemented_interface = subgraphs.strings.intern(implemented_interface.node.as_str());
                    subgraphs.push_interface_impl(definition_name, implemented_interface);
                }

                fields::ingest_fields(
                    definition_id,
                    &interface.fields,
                    federation_directives_matcher,
                    subgraphs,
                );
            }
            ast::TypeKind::Object(object_type) => {
                let definition_id = subgraphs.definition_by_name(&definition.node.name.node, subgraph_id);
                let definition_name = subgraphs.walk(definition_id).name().id;

                for implemented_interface in &object_type.implements {
                    let implemented_interface = subgraphs.strings.intern(implemented_interface.node.as_str());
                    subgraphs.push_interface_impl(definition_name, implemented_interface);
                }

                fields::ingest_fields(
                    definition_id,
                    &object_type.fields,
                    federation_directives_matcher,
                    subgraphs,
                );
            }
            _ => (),
        }
    }
}
