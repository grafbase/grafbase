//! Implement the Relation Engine
use super::model_directive::MODEL_DIRECTIVE;
use super::visitor::{Visitor, VisitorContext};
use crate::registry::add_create_mutation;
use crate::registry::add_list_query_paginated;
use crate::registry::add_remove_query;
use crate::utils::is_id_type_and_non_nullable;
use crate::utils::is_modelized_node;
use crate::utils::to_base_type_str;
use crate::utils::to_lower_camelcase;
use async_graphql::indexmap::{map::Entry, IndexMap};
use async_graphql::registry::relations::MetaRelation;
use async_graphql::registry::resolvers::context_data::ContextDataResolver;
use async_graphql::registry::resolvers::dynamo_querying::DynamoResolver;
use async_graphql::registry::MetaField;
use async_graphql::registry::MetaInputValue;
use async_graphql::registry::MetaType;
use async_graphql::registry::{
    resolvers::Resolver, resolvers::ResolverType, transformers::Transformer, variables::VariableResolveDefinition,
};
use async_graphql_parser::types::{Type, TypeKind};
use if_chain::if_chain;

/// Implement the Relation Engine
///
/// We need to define the relation before hand, to do that we have two mecanism
/// working to define relation:
///
/// - Implicit: By having an explicit relation on two modelized node.
/// - Explicit: By having the relation defined by the `@relation` directive
///
/// A relation can only exist between two nodes.
///
/// # Algorithm
///
/// For each modelized node, we go into each fields, for each field:
///
/// -> We pass into the field
/// --> If modelized
/// --> Attribute a relation name based on those two types.
/// --> Store it into the VisitorContext
/// --> (Store it into the generated type, need dynaql work)
pub struct RelationEngine;

pub const RELATION_DIRECTIVE: &str = "relation";

impl<'a> Visitor<'a> for RelationEngine {
    fn directives(&self) -> String {
        r#"
        directive @relation(
          """
          The name of the relation
          """
          name: String!
        ) on FIELD_DEFINITION
        "#
        .to_string()
    }

    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a async_graphql::Positioned<async_graphql_parser::types::TypeDefinition>,
    ) {
        let directives = &type_definition.node.directives;
        if_chain! {
            // We do check if it's a modelized node
            // TODO: Create an abstraction over it
            let _ = directives.iter().find(|directive| directive.node.name.node == MODEL_DIRECTIVE);
            if let TypeKind::Object(object) = &type_definition.node.kind;
            // We do check if it's a modelized node
            then {
                let type_name = type_definition.node.name.node.to_string();
                // We iterate over fields that reprensent a relation to check than
                for (field, _) in object.fields.iter().filter_map(|field| is_modelized_node(&ctx.types, &field.node.ty.node).map(|ty| (field, ty))) {
                    let name = field.node.directives.iter().find(|directive| directive.node.name.node == RELATION_DIRECTIVE).and_then(|dir| dir.node.get_argument("name")).map(|name| name.node.to_string());
                    let relation = MetaRelation::new(name, &Type::new(&type_name).expect("Shouldn't fail"), &field.node.ty.node);
                    match ctx.relations.entry(relation.name.clone()) {
                        Entry::Vacant(vac) => {
                            vac.insert(relation);
                        }
                        Entry::Occupied(mut oqp) => {
                            oqp.get_mut().with(relation);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RelationEngine;
    use crate::rules::visitor::{visit, VisitorContext};
    use async_graphql_parser::parse_schema;
    use insta::assert_debug_snapshot;
    use serde_json as _;

    #[test]
    fn one_to_one_relation_monodirectional() {
        let schema = r#"
            type Author @model {
                id: ID!
            }

            type Post @model {
                id: ID!
                publishedBy: Author
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1 as usize, "Should have only one relation");
        assert_eq!(
            ctx.relations.iter().next().unwrap().1.birectional,
            false,
            "Should be monodirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn one_to_one_relation_bidirectionnal() {
        let schema = r#"
            type Author @model {
                id: ID!
                published: Post
            }

            type Post @model {
                id: ID!
                publishedBy: Author
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1 as usize, "Should have only one relation");
        assert_eq!(
            ctx.relations.iter().next().unwrap().1.birectional,
            true,
            "Should be bidirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn one_to_many_relation_monodirectional_1() {
        let schema = r#"
            type Author @model {
                id: ID!
            }

            type Post @model {
                id: ID!
                publishedBy: [Author]
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1 as usize, "Should have only one relation");
        assert_eq!(
            ctx.relations.iter().next().unwrap().1.birectional,
            false,
            "Should be monodirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn one_to_many_relation_monodirectional_2() {
        let schema = r#"
            type Author @model {
                id: ID!
                posts: [Post]
            }

            type Post @model {
                id: ID!
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1 as usize, "Should have only one relation");
        assert_eq!(
            ctx.relations.iter().next().unwrap().1.birectional,
            false,
            "Should be monodirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn one_to_many_relation_bidirectional_1() {
        let schema = r#"
            type Author @model {
                id: ID!
                post: Post!
            }

            type Post @model {
                id: ID!
                publishedBy: [Author]
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1 as usize, "Should have only one relation");
        assert_eq!(
            ctx.relations.iter().next().unwrap().1.birectional,
            true,
            "Should be bidirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn one_to_many_relation_bidirectional_2() {
        let schema = r#"
            type Author @model {
                id: ID!
                posts: [Post]
            }

            type Post @model {
                id: ID!
                author: Author!
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1 as usize, "Should have only one relation");
        assert_eq!(
            ctx.relations.iter().next().unwrap().1.birectional,
            true,
            "Should be bidirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn many_to_many_relation_monodirectional() {
        let schema = r#"
            type Author @model {
                id: ID!
                posts: [Post!]
            }

            type Post @model {
                id: ID!
                publishedBy: [Author!]
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
        assert_eq!(ctx.relations.len(), 1 as usize, "Should have only one relation");
        assert_eq!(
            ctx.relations.iter().next().unwrap().1.birectional,
            true,
            "Should be bidirectional"
        );
        assert_debug_snapshot!(&ctx.relations);
    }

    #[test]
    fn multiple_relations() {
        let schema = r#"
            type Author @model {
                id: ID!
                postsToday: [Post!]
                postsYesterday: [Post!]
            }

            type Post @model {
                id: ID!
                publishedBy: [Author!]
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut RelationEngine, &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
    }
}
