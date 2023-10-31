//! This is a separate module because we want to use only the public API of [Subgraphs] and avoid
//! mixing GraphQL parser logic and types with our internals.

mod field;
mod object;
mod schema_definitions;

use self::{field::*, schema_definitions::*};
use crate::{
    subgraphs::{DefinitionKind, SubgraphId},
    Subgraphs,
};
use async_graphql_parser::types as ast;

pub(crate) fn ingest_subgraph(
    document: &ast::ServiceDocument,
    name: &str,
    subgraphs: &mut Subgraphs,
) {
    let subgraph_id = subgraphs.push_subgraph(name);

    let federation_directives_matcher = ingest_schema_definitions(document);

    ingest_top_level_definitions(
        subgraph_id,
        document,
        subgraphs,
        &federation_directives_matcher,
    );

    ingest_definition_bodies(subgraph_id, document, subgraphs);
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
                    ast::TypeKind::Object(object_type) => {
                        let definition_id = subgraphs.push_definition(
                            subgraph_id,
                            type_name,
                            DefinitionKind::Object,
                        );
                        object::ingest_directives(
                            definition_id,
                            &type_definition.node,
                            subgraphs,
                            federation_directives_matcher,
                        );
                        let object_is_shareable = subgraphs.walk(definition_id).is_shareable();

                        for field in &object_type.fields {
                            let is_shareable = object_is_shareable
                                || field.node.directives.iter().any(|directive| {
                                    federation_directives_matcher
                                        .is_shareable(directive.node.name.node.as_str())
                                });
                            let type_id = subgraphs.intern_field_type(&field.node.ty.node);
                            let field_id = subgraphs.push_field(
                                definition_id,
                                &field.node.name.node,
                                type_id,
                                is_shareable,
                            );

                            ingest_field_arguments(field_id, &field.node.arguments, subgraphs);
                        }
                    }
                    ast::TypeKind::Interface(_interface_type) => {
                        subgraphs.push_definition(
                            subgraph_id,
                            type_name,
                            DefinitionKind::Interface,
                        );
                    }
                    ast::TypeKind::Union(_) => {
                        subgraphs.push_definition(subgraph_id, type_name, DefinitionKind::Union);
                    }
                    ast::TypeKind::InputObject(_) => {
                        subgraphs.push_definition(
                            subgraph_id,
                            type_name,
                            DefinitionKind::InputObject,
                        );
                    }
                    _ => (), // TODO
                }
            }
            ast::TypeSystemDefinition::Schema(_) => (),
            ast::TypeSystemDefinition::Directive(_) => (),
        }
    }
}

fn ingest_definition_bodies(
    subgraph_id: SubgraphId,
    document: &ast::ServiceDocument,
    subgraphs: &mut Subgraphs,
) {
    let type_definitions = document.definitions.iter().filter_map(|def| match def {
        ast::TypeSystemDefinition::Type(ty) => Some(ty),
        _ => None,
    });

    for definition in type_definitions {
        match &definition.node.kind {
            ast::TypeKind::Union(union) => {
                let union_id =
                    subgraphs.definition_by_name(&definition.node.name.node, subgraph_id);

                for member in &union.members {
                    let member_id = subgraphs.definition_by_name(&member.node, subgraph_id);
                    subgraphs.push_union_member(union_id, member_id);
                }
            }
            ast::TypeKind::InputObject(input_object) => {
                let definition_id =
                    subgraphs.definition_by_name(&definition.node.name.node, subgraph_id);
                for field in &input_object.fields {
                    let ty = subgraphs.intern_field_type(&field.node.ty.node);
                    subgraphs.push_field(definition_id, &field.node.name.node, ty, false);
                }
            }
            _ => (),
        }
    }
}
