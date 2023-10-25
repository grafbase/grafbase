//! This is a separate module because we want to use only the public API of [Subgraphs] and avoid
//! mixing GraphQL parser logic and types with our internals.

use crate::{subgraphs::DefinitionKind, Subgraphs};
use async_graphql_parser::types as ast;

pub(crate) fn ingest_subgraph(
    document: &ast::ServiceDocument,
    name: &str,
    subgraphs: &mut Subgraphs,
) {
    let subgraph_id = subgraphs.push_subgraph(name);

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

                        for field in &object_type.fields {
                            let type_name = resolve_field_type(&field.node.ty.node.base);
                            subgraphs.push_field(
                                definition_id,
                                &field.node.name.node,
                                type_name,
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
            ast::TypeSystemDefinition::Schema(_) => (), // TODO
            ast::TypeSystemDefinition::Directive(_) => (), // TODO
        }
    }
}

fn resolve_field_type(base_type: &ast::BaseType) -> &str {
    match base_type {
        ast::BaseType::Named(name) => name,
        ast::BaseType::List(inner) => resolve_field_type(&inner.base),
    }
}
