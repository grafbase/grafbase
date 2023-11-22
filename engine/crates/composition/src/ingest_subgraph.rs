//! This is a separate module because we want to use only the public API of [Subgraphs] and avoid
//! mixing GraphQL parser logic and types with our internals.

mod enums;
mod field;
mod nested_key_fields;
mod object;
mod schema_definitions;

use self::{nested_key_fields::ingest_nested_key_fields, schema_definitions::*};
use crate::{
    subgraphs::{self, DefinitionKind, SubgraphId},
    Subgraphs,
};
use async_graphql_parser::types as ast;

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
                match &type_definition.node.kind {
                    ast::TypeKind::Object(_) => {
                        let definition_id = subgraphs.push_definition(subgraph_id, type_name, DefinitionKind::Object);
                        object::ingest_directives(
                            definition_id,
                            &type_definition.node,
                            subgraphs,
                            federation_directives_matcher,
                        );
                    }
                    ast::TypeKind::Interface(_interface_type) => {
                        subgraphs.push_definition(subgraph_id, type_name, DefinitionKind::Interface);
                    }
                    ast::TypeKind::Union(_) => {
                        subgraphs.push_definition(subgraph_id, type_name, DefinitionKind::Union);
                    }
                    ast::TypeKind::InputObject(_) => {
                        subgraphs.push_definition(subgraph_id, type_name, DefinitionKind::InputObject);
                    }

                    ast::TypeKind::Scalar => {
                        subgraphs.push_definition(subgraph_id, type_name, DefinitionKind::Scalar);
                    }

                    ast::TypeKind::Enum(enum_type) => {
                        let definition_id = subgraphs.push_definition(subgraph_id, type_name, DefinitionKind::Enum);
                        enums::ingest_enum(definition_id, enum_type, subgraphs);
                    }
                }
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
                for field in &input_object.fields {
                    let field_type = subgraphs.intern_field_type(&field.node.ty.node);
                    subgraphs
                        .push_field(subgraphs::FieldIngest {
                            parent_definition_id: definition_id,
                            field_name: &field.node.name.node,
                            field_type,
                            is_shareable: false,
                            is_external: false,
                            provides: None,
                            requires: None,
                        })
                        .unwrap();
                }
            }
            ast::TypeKind::Interface(interface) => {
                let definition_id = subgraphs.definition_by_name(&definition.node.name.node, subgraph_id);
                let definition_name = subgraphs.walk(definition_id).name().id;

                for implemented_interface in &interface.implements {
                    let implemented_interface = subgraphs.strings.intern(implemented_interface.node.as_str());
                    subgraphs.push_interface_impl(definition_name, implemented_interface);
                }

                for field in &interface.fields {
                    let field_type = subgraphs.intern_field_type(&field.node.ty.node);
                    subgraphs
                        .push_field(subgraphs::FieldIngest {
                            parent_definition_id: definition_id,
                            field_name: &field.node.name.node,
                            field_type,
                            is_shareable: false,
                            is_external: false,
                            provides: None,
                            requires: None,
                        })
                        .unwrap();
                }
            }
            ast::TypeKind::Object(object_type) => {
                let definition_id = subgraphs.definition_by_name(&definition.node.name.node, subgraph_id);
                let definition_name = subgraphs.walk(definition_id).name().id;

                for implemented_interface in &object_type.implements {
                    let implemented_interface = subgraphs.strings.intern(implemented_interface.node.as_str());
                    subgraphs.push_interface_impl(definition_name, implemented_interface);
                }

                object::ingest_fields(definition_id, object_type, federation_directives_matcher, subgraphs);
            }
            _ => (),
        }
    }
}
