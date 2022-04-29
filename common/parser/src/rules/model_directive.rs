//! Implement the model directive
//!
//! When a @model directive is present for a type, we generate the associated type into the
//! registry and generate the CRUDL configuration for this type.
//!
//! Flow:
//!  -> When there is a @model directive on a type
//!  -> Must be an ObjectType
//!  -> Must have primitives
//!  -> Must have a non_nullable ID type
//!
//! Then:
//!  -> Create the ObjectType
//!  -> Create the ReadById Query
//!  -> Create the Create Mutation
//!
//! TODO: Should have either: an ID or a PK

use super::visitor::{Visitor, VisitorContext};
use crate::registry::add_create_mutation;
use crate::registry::add_remove_query;
use crate::utils::is_id_type_and_non_nullable;
use crate::utils::is_modelized_node;
use crate::utils::to_base_type_str;
use async_graphql::indexmap::IndexMap;
use async_graphql::registry::resolvers::context_data::ContextDataResolver;
use async_graphql::registry::resolvers::dynamo_querying::DynamoResolver;
use async_graphql::registry::MetaField;
use async_graphql::registry::MetaInputValue;
use async_graphql::registry::MetaType;
use async_graphql::registry::{
    resolvers::Resolver, resolvers::ResolverType, transformers::Transformer, variables::VariableResolveDefinition,
};
use async_graphql_parser::types::TypeKind;
use if_chain::if_chain;

pub struct ModelDirective;

pub const MODEL_DIRECTIVE: &str = "model";

impl<'a> Visitor<'a> for ModelDirective {
    fn directives(&self) -> String {
        r#"
        directive @model on OBJECT
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
                    if directives.iter().any(|directive| directive.node.name.node == MODEL_DIRECTIVE);
                    if let TypeKind::Object(object) = &type_definition.node.kind;
                    if let Some(id_field) = object.fields.iter().find(|x| is_id_type_and_non_nullable(&x.node));
                    then {
                        let type_name = type_definition.node.name.node.to_string();
                        // If it's a modeled Type, we create the associated type into the registry.
                        // Without more data, we infer it's from our modelization.
                        ctx.registry.get_mut().create_type(&mut |_| MetaType::Object {
                            name: type_name.clone(),
                            description: type_definition.node.description.clone().map(|x| x.node),
                            fields: {
                                let mut fields = IndexMap::new();
                                for field in &object.fields {
                                    let name = field.node.name.node.to_string();
                                    let ty = is_modelized_node(&ctx.types, &field.node.ty.node);
                                    let is_node = ty.and_then(|x| {
                                        match &x.node.kind {
                                            TypeKind::Object(obj) => Some(obj),
                                            _ => None
                                        }
                                    }).and_then(|obj| {
                                        obj.fields.iter().find(|field| is_modelized_node(&ctx.types, &field.node.ty.node).is_some())
                                    }).is_some();
                                    let is_edge = ty.is_some();
                                    let transforms = if is_edge {
                                        None
                                    } else {
                                        Some(vec![Transformer::DynamoSelect {
                                            property: if name == "id" {
                                                "__sk".to_string()
                                            } else {
                                                name.clone()
                                            },
                                        }])
                                    };

                                    let resolve = if is_edge {
                                        Some(Resolver {
                                            id: Some(format!("{}_edge_resolver", type_name.to_lowercase())),
                                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::Edge {
                                                key: to_base_type_str(&field.node.ty.node.base),
                                                is_node,
                                            }),
                                        })
                                    } else {
                                        Some(Resolver {
                                            id: None,
                                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::LocalKey { key: type_name.to_string() }),
                                        })
                                    };

                                    fields.insert(name.clone(), MetaField {
                                        name,
                                        description: field.node.description.clone().map(|x| x.node),
                                        args: Default::default(),
                                        ty: field.node.ty.clone().node.to_string(),
                                        deprecation: Default::default(),
                                        cache_control: Default::default(),
                                        external: false,
                                        requires: None,
                                        provides: None,
                                        visible: None,
                                        compute_complexity: None,
                                        resolve,
                                        is_edge: if is_edge {
                                            Some(to_base_type_str(&field.node.ty.node.base))
                                        } else {
                                            None
                                        },
                                        transforms,
                                    });
                                };
                                fields
                            },
                            cache_control: async_graphql::CacheControl {
                                public: true,
                                max_age: 0usize,
                            },
                            extends: false,
                            keys: None,
                            visible: None,
                            is_subscription: false,
                            is_node: true,
                            rust_typename: type_name.clone(),
                        }, &type_name, &type_name);

                        ctx.queries.push(MetaField {
                            // byID query
                            name: type_name.to_lowercase(),
                            description: Some(format!("Get a {} by ID", type_name)),
                            args: {
                                let mut args = IndexMap::new();
                                args.insert(
                                    "id".to_owned(),
                                    MetaInputValue {
                                        name: "id".to_owned(),
                                        ty: "ID!".to_string(),
                                        visible: None,
                                        description: Some(format!("{} id", type_name)),
                                        is_secret: false,
                                        default_value: None,
                                    },
                                );
                                args
                            },
                            ty: type_name.clone(),
                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                            cache_control: async_graphql::CacheControl {
                                public: true,
                                max_age: 0usize,
                            },
                            external: false,
                            provides: None,
                            requires: None,
                            visible: None,
                            compute_complexity: None,
                            is_edge: None,
                            resolve: Some(Resolver {
                                id: Some(format!("{}_resolver", type_name.to_lowercase())),
                                // TODO: Should be defined as a ResolveNode
                                // Single entity
                                r#type: ResolverType::DynamoResolver(DynamoResolver::QueryPKSK {
                                    pk: VariableResolveDefinition::InputTypeName("id".to_owned()),
                                    sk: VariableResolveDefinition::InputTypeName("id".to_owned()),
                                })
                            }),
                            transforms: None,
                        });

                        ctx.queries.push(MetaField {
                            name: format!("{}Collection",type_name.to_lowercase()),
                            description: Some(format!("Unpaginated query to fetch the whole list of `{}`.", type_name)),
                            args: IndexMap::new(),
                            ty: format!("[{}]", type_name.clone()),
                            deprecation: async_graphql::registry::Deprecation::NoDeprecated,
                            cache_control: async_graphql::CacheControl {
                                public: true,
                                max_age: 0usize,
                            },
                            external: false,
                            provides: None,
                            requires: None,
                            visible: None,
                            compute_complexity: None,
                            is_edge: None,
                            resolve: Some(Resolver {
                                id: Some(format!("{}_resolver", type_name.to_lowercase())),
                                // Multiple entities
                                r#type: ResolverType::DynamoResolver(DynamoResolver::ListResultByType {
                                    r#type: VariableResolveDefinition::DebugString(type_name.clone()),
                                })
                            }),
                            transforms: None,
                        });

                        add_create_mutation(ctx, object, &id_field.node, &type_name);
                        add_remove_query(ctx, &id_field.node, &type_name)
                    }
                }
    }
}
