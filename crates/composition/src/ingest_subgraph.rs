//! This is a separate module because we want to use only the public API of [Subgraphs] and avoid
//! mixing GraphQL parser logic and types with our internals.

mod schema_definitions;

use self::schema_definitions::*;
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

                        let object_is_shareable = type_definition.node.directives.iter().any(|d| {
                            federation_directives_matcher.is_shareable(d.node.name.node.as_str())
                        });

                        for field in &object_type.fields {
                            let is_shareable = object_is_shareable
                                || field.node.directives.iter().any(|directive| {
                                    federation_directives_matcher
                                        .is_shareable(directive.node.name.node.as_str())
                                });
                            let type_name = resolve_field_type(&field.node.ty.node.base);
                            subgraphs.push_field(
                                definition_id,
                                &field.node.name.node,
                                type_name,
                                is_shareable,
                            );
                        }
                    }
                    ast::TypeKind::Interface(_interface_type) => {
                        let _definition_id = subgraphs.push_definition(
                            subgraph_id,
                            type_name,
                            DefinitionKind::Interface,
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

fn resolve_field_type(base_type: &ast::BaseType) -> &str {
    match base_type {
        ast::BaseType::Named(name) => name,
        ast::BaseType::List(inner) => resolve_field_type(&inner.base),
    }
}
