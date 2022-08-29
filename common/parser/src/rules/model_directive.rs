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

use super::relations::generate_metarelation;
use super::visitor::{Visitor, VisitorContext};
use crate::registry::add_list_query_paginated;
use crate::registry::add_remove_query;
use crate::registry::{add_create_mutation, add_update_mutation};
use crate::utils::is_modelized_node;
use crate::utils::to_base_type_str;
use crate::utils::to_lower_camelcase;
use crate::utils::{is_id_type_and_non_nullable, pagination_arguments};
use case::CaseExt;
use dynaql::indexmap::IndexMap;
use dynaql::registry::resolvers::context_data::ContextDataResolver;
use dynaql::registry::resolvers::dynamo_querying::DynamoResolver;
use dynaql::registry::{is_array_basic_type, MetaType};
use dynaql::registry::{
    resolvers::Resolver, resolvers::ResolverType, transformers::Transformer, variables::VariableResolveDefinition,
};
use dynaql::registry::{Constraint, MetaField};
use dynaql::registry::{ConstraintType, MetaInputValue};
use dynaql::Operations;
use dynaql_parser::types::{Type, TypeKind};
use if_chain::if_chain;

pub struct ModelDirective;

pub const MODEL_DIRECTIVE: &str = "model";
pub const UNIQUE_DIRECTIVE: &str = "unique";

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
        type_definition: &'a dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
    ) {
        let directives = &type_definition.node.directives;
        if_chain! {
            if directives.iter().any(|directive| directive.node.name.node == MODEL_DIRECTIVE);
            if let TypeKind::Object(object) = &type_definition.node.kind;
            if let Some(id_field) = object.fields.iter().find(|x| is_id_type_and_non_nullable(&x.node));
            then {
                let type_name = type_definition.node.name.node.to_string();
                let mut connection_edges = Vec::new();
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
                            let relation = ty.map(|_ty| {
                                generate_metarelation(&type_definition.node, &field.node)
                            });

                            let is_edge = relation.as_ref().map(|x| {
                                let target_ty = Type::new(x.relation.1.as_str()).expect("shouldn't fail");
                                is_modelized_node(&ctx.types, &target_ty).is_some()
                            }).unwrap_or_default();

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

                            let is_expecting_array = is_array_basic_type(&field.node.ty.to_string());
                            let relation_array = relation.is_some() && is_expecting_array;

                            let resolve = match (is_edge, &relation) {
                                (true, Some(relation)) => Some(Resolver {
                                    id: Some(format!("{}_edge_resolver", relation.relation.0.to_lowercase())),
                                    r#type: ResolverType::ContextDataResolver(if relation_array {
                                        ContextDataResolver::EdgeArray {
                                            key: relation.relation.0.clone(),
                                            relation_name: relation.name.clone(),
                                            expected_ty: to_base_type_str(&field.node.ty.node.base),
                                        }
                                    } else {
                                        ContextDataResolver::SingleEdge {
                                            key: relation.relation.0.clone(),
                                            relation_name: relation.name.clone(),
                                        }
                                    }),
                                }),
                                (false, Some(relation)) => Some(Resolver {
                                    id: None,
                                    r#type: ResolverType::ContextDataResolver(ContextDataResolver::LocalKey { key: relation.name.clone() }),
                                }),
                                (false, None) => Some(Resolver {
                                    id: None,
                                    r#type: ResolverType::ContextDataResolver(ContextDataResolver::LocalKey { key: type_name.to_string() }),
                                }),
                                _ => unreachable!("Can't happen yet"),
                            };

                            fields.insert(name.clone(), MetaField {
                                name,
                                description: field.node.description.clone().map(|x| x.node),
                                args: if relation_array {
                                    pagination_arguments()
                                } else {
                                    Default::default()
                                },
                                ty: if relation_array {
                                    format!("{}Connection", to_base_type_str(&field.node.ty.node.base).to_camel())
                                } else {
                                    field.node.ty.clone().node.to_string()
                                },
                                deprecation: Default::default(),
                                cache_control: Default::default(),
                                external: false,
                                requires: None,
                                provides: None,
                                visible: None,
                                compute_complexity: None,
                                resolve,
                                edges: if is_edge {
                                    let edge_type = to_base_type_str(&field.node.ty.node.base);
                                    connection_edges.push(edge_type.clone());
                                    vec![edge_type]
                                } else {
                                    Vec::new()
                                },
                                relation,
                                transforms,
                                required_operation: None,
                            });
                        };
                        fields
                    },
                    cache_control: dynaql::CacheControl {
                        public: true,
                        max_age: 0usize,
                    },
                    extends: false,
                    keys: None,
                    visible: None,
                    is_subscription: false,
                    is_node: true,
                    rust_typename: type_name.clone(),
                    constraints: object.fields
                        .iter()
                        .filter_map(|field| field
                            .node
                            .directives
                            .iter()
                            .find(|directive| directive.node.name.node == UNIQUE_DIRECTIVE)
                            .map(|_| Constraint {
                                field: field.node.name.to_string(),
                                r#type: ConstraintType::Unique,
                            }))
                        .collect(),
                }, &type_name, &type_name);

                ctx.queries.push(MetaField {
                    // byID query
                    name: to_lower_camelcase(&type_name),
                    description: Some(format!("Get {} by ID", type_name)),
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
                    deprecation: dynaql::registry::Deprecation::NoDeprecated,
                    cache_control: dynaql::CacheControl {
                        public: true,
                        max_age: 0usize,
                    },
                    external: false,
                    provides: None,
                    requires: None,
                    visible: None,
                    compute_complexity: None,
                    edges: Vec::new(),
                    relation: None,
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
                    required_operation: Some(Operations::GET),
                });

                add_create_mutation(ctx, &type_definition.node, object, &type_name);
                add_update_mutation(ctx, &type_definition.node, object, &type_name);

                add_list_query_paginated(ctx, &type_name, connection_edges);
                add_remove_query(ctx, &id_field.node, &type_name)
            }
        }
    }
}
