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
use crate::registry::add_remove_mutation;
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
use dynaql::{Operations, Positioned};
use dynaql_parser::types::{FieldDefinition, Type, TypeKind};
use if_chain::if_chain;

pub struct ModelDirective;

pub const MODEL_DIRECTIVE: &str = "model";
pub const UNIQUE_DIRECTIVE: &str = "unique";

fn insert_metadata_field(
    fields: &mut IndexMap<String, MetaField>,
    type_name: &str,
    field_name: &str,
    description: Option<String>,
    ty: &str,
    dynamo_property_name: &str,
) -> Option<MetaField> {
    fields.insert(
        field_name.to_owned(),
        MetaField {
            name: field_name.to_owned(),
            description,
            args: Default::default(),
            ty: ty.to_owned(),
            deprecation: Default::default(),
            cache_control: Default::default(),
            external: false,
            requires: None,
            provides: None,
            visible: None,
            compute_complexity: None,
            resolve: Some(Resolver {
                id: None,
                r#type: ResolverType::ContextDataResolver(ContextDataResolver::LocalKey {
                    key: type_name.to_string(),
                }),
            }),
            edges: Vec::new(),
            transforms: Some(vec![Transformer::DynamoSelect {
                property: dynamo_property_name.to_owned(),
            }]),
            relation: None,
            required_operation: None,
        },
    )
}

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
            then {
               if !object.fields.iter().any(|x| is_id_type_and_non_nullable(&x.node)) {
                    let name_ty = &type_definition.node.name.node;
                    ctx.report_error(vec![type_definition.pos], format!("\"{name_ty}\" doesn't implement @model properly, please add a non-nullable ID field."));
                    return;
                };
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
                        insert_metadata_field(&mut fields, &type_name, "updatedAt", Some("when the model was updated".to_owned()), "DateTime!", "__updated_at");
                        insert_metadata_field(&mut fields, &type_name, "createdAt", Some("when the model was created".to_owned()), "DateTime!", "__created_at");

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

                let unique_fields: Vec<&Positioned<FieldDefinition>> = object.fields
                    .iter()
                    .filter(|field| field
                        .node
                        .directives
                        .iter()
                        .any(|directive| directive.node.name.node == UNIQUE_DIRECTIVE)
                    )
                    .collect();

                    let one_of_type_name = format!("{}ByInput", type_name);

                    ctx.registry.get_mut().create_type(&mut |_| MetaType::InputObject  {
                        name: one_of_type_name.clone(),
                        description: type_definition.node.description.clone().map(|description| description.node),
                        visible: None,
                        rust_typename: one_of_type_name.clone(),
                        input_fields: {
                            let mut input_fields = IndexMap::new();
                            input_fields.insert(
                                "id".to_string(),
                                MetaInputValue {
                                    name: "id".to_string(),
                                    description: None,
                                    ty: "ID".to_string(),
                                    default_value: None,
                                    visible: None,
                                    is_secret: false
                                }
                            );
                            for field in &unique_fields {
                                input_fields.insert(
                                    field.node.name.to_string(),
                                    MetaInputValue {
                                        name: field.node.name.to_string(),
                                        description: None,
                                        ty: field.node.ty.to_string().trim_end_matches('!').to_string(),
                                        default_value: None,
                                        visible: None,
                                        is_secret: false
                                    }
                                );
                            }
                            input_fields
                        },
                        oneof: true,
                    }, &one_of_type_name, &one_of_type_name);

                ctx.queries.push(MetaField {
                    // "by" query
                    name: to_lower_camelcase(&type_name),
                    description: Some(format!("Query a single {} by an ID or a unique field", type_name)),
                    args: {
                        let mut args = IndexMap::new();
                        args.insert(
                            "by".to_owned(),
                            MetaInputValue {
                                name: "by".to_owned(),
                                ty: format!("{}ByInput!", type_name),
                                visible: None,
                                description: Some(format!("The field and value by which to query the {}", type_name)),
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
                        r#type: ResolverType::DynamoResolver(DynamoResolver::QueryBy {
                            by: VariableResolveDefinition::InputTypeName("by".to_owned()),
                        })
                    }),
                    transforms: None,
                    required_operation: Some(Operations::GET),
                });

                add_create_mutation(ctx, &type_definition.node, object, &type_name);
                add_update_mutation(ctx, &type_definition.node, object, &type_name);

                add_list_query_paginated(ctx, &type_name, connection_edges);
                add_remove_mutation(ctx, &type_name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ModelDirective;
    use crate::rules::visitor::{visit, VisitorContext};
    use dynaql_parser::parse_schema;
    use serde_json as _;

    #[test]
    fn should_error_when_defining_an_invalid_model() {
        let schema = r#"
            type Product @model {
                test: String!
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut ModelDirective, &mut ctx, &schema);

        assert!(!ctx.errors.is_empty(), "shouldn't be empty");
        assert_eq!(ctx.errors.len(), 1, "should have one error");
    }

    #[test]
    fn should_not_error_when_id() {
        let schema = r#"
            type Product @model {
                id: ID!
                test: String!
            }
            "#;

        let schema = parse_schema(schema).expect("");

        let mut ctx = VisitorContext::new(&schema);
        visit(&mut ModelDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "should be empty");
    }
}
