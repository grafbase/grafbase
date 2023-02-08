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

use case::CaseExt;
use dynaql::registry::plan::SchemaPlan;
use dynaql::registry::resolvers::custom::CustomResolver;
use if_chain::if_chain;

use dynaql::indexmap::IndexMap;
use dynaql::registry::resolvers::context_data::ContextDataResolver;
use dynaql::registry::resolvers::dynamo_querying::DynamoResolver;
use dynaql::registry::scalars::{DateTimeScalar, IDScalar, SDLDefinitionScalar};
use dynaql::registry::MetaField;
use dynaql::registry::MetaInputValue;
use dynaql::registry::{is_array_basic_type, MetaType};
use dynaql::registry::{
    resolvers::Resolver, resolvers::ResolverType, transformers::Transformer, variables::VariableResolveDefinition,
};
use dynaql::{AuthConfig, Operations, Positioned};
use dynaql_parser::types::{BaseType, FieldDefinition, ObjectType, Type, TypeDefinition, TypeKind};
use std::borrow::Cow;
use std::collections::HashMap;

use crate::registry::add_remove_mutation;
use crate::registry::generate_pagination_args;
use crate::registry::names::MetaNames;
use crate::registry::{add_mutation_create, add_mutation_update};
use crate::registry::{add_query_paginated_collection, add_query_search};
use crate::utils::to_base_type_str;
use crate::utils::to_lower_camelcase;

use super::auth_directive::AuthDirective;
use super::directive::Directive;
use super::relations::RelationEngine;
use super::resolver_directive::ResolverDirective;
use super::search_directive::SEARCH_DIRECTIVE;
use super::unique_directive::UniqueDirective;
use super::visitor::{Visitor, VisitorContext};

pub struct ModelDirective;

pub const RESERVED_FIELD_ID: &str = "id";
pub const RESERVED_FIELD_CREATED_AT: &str = "createdAt";
pub const RESERVED_FIELD_UPDATED_AT: &str = "updatedAt";
pub const RESERVED_FIELDS: [&str; 3] = [RESERVED_FIELD_ID, RESERVED_FIELD_UPDATED_AT, RESERVED_FIELD_CREATED_AT];
pub const MODEL_DIRECTIVE: &str = "model";

impl ModelDirective {
    pub fn is_not_reserved_field(field: &Positioned<FieldDefinition>) -> bool {
        !RESERVED_FIELDS.contains(&field.node.name.node.as_str())
    }

    pub fn is_model(ctx: &'_ VisitorContext<'_>, ty: &Type) -> bool {
        Self::get_model_type_definition(ctx, &ty.base).is_some()
    }

    pub fn get_model_type_definition<'a, 'b>(
        ctx: &'a VisitorContext<'b>,
        base: &BaseType,
    ) -> Option<&'a Cow<'b, Positioned<TypeDefinition>>> {
        match base {
            BaseType::Named(name) => ctx.types.get(name.as_ref()).and_then(|ty| {
                if_chain!(
                    if let TypeKind::Object(_) = &ty.node.kind;
                    if ty.node.directives.iter().any(|directive| directive.node.name.node == MODEL_DIRECTIVE);
                    then { Some(ty) }
                    else { None }
                )
            }),
            BaseType::List(list) => Self::get_model_type_definition(ctx, &list.base),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn insert_metadata_field(
    fields: &mut IndexMap<String, MetaField>,
    type_name: &str,
    field_name: &str,
    description: Option<String>,
    ty: &str,
    dynamo_property_name: &str,
    plan_field: &str,
    auth: Option<&AuthConfig>,
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
            plan: Some(SchemaPlan::projection(vec![plan_field.to_string()])),
            edges: Vec::new(),
            transformer: Some(Transformer::DynamoSelect {
                property: dynamo_property_name.to_owned(),
            }),
            relation: None,
            required_operation: None,
            auth: auth.cloned(),
        },
    )
}

impl Directive for ModelDirective {
    fn definition() -> String {
        r#"
        directive @model on OBJECT
        "#
        .to_string()
    }
}

impl<'a> Visitor<'a> for ModelDirective {
    fn enter_type_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        type_definition: &'a dynaql::Positioned<dynaql_parser::types::TypeDefinition>,
    ) {
        if !&type_definition
            .node
            .directives
            .iter()
            .any(|directive| directive.node.name.node == MODEL_DIRECTIVE)
        {
            return;
        }
        if let TypeKind::Object(object) = &type_definition.node.kind {
            let type_name = MetaNames::model(&type_definition.node);
            if has_any_invalid_reserved_fields(ctx, &type_name, object) {
                return;
            }

            //
            // AUTHORIZATION
            //
            let model_auth = match AuthDirective::parse(ctx, &type_definition.node.directives, false) {
                Ok(auth) => auth,
                Err(err) => {
                    ctx.report_error(err.locations, err.message);
                    None
                }
            };
            // Do this here since ctx can't be borrowed mutably twice inside ctx.registry.get_mut() below
            let field_auth = object.fields.iter().fold(HashMap::new(), |mut map, field| {
                let name = field.node.name.node.to_string();
                let auth = match AuthDirective::parse(ctx, &field.node.directives, false) {
                    Ok(auth) => auth,
                    Err(err) => {
                        ctx.report_error(err.locations, err.message);
                        None
                    }
                }
                .or_else(|| model_auth.clone()); // Fall back to model auth if field auth is not configured
                map.insert(name, auth);
                map
            });

            let unique_directives = object
                .fields
                .iter()
                .filter_map(|field| UniqueDirective::parse(ctx, object, &type_name, field))
                .collect::<Vec<_>>();

            // Add typename schema
            let schema_id = ctx.new_schema_id(&type_name);

            // Add typename schema
            let schema_id = ctx.get_schema_id(&type_name);

            //
            // CREATE ACTUAL TYPE
            //
            let mut connection_edges = Vec::new();
            // If it's a modeled Type, we create the associated type into the registry.
            // Without more data, we infer it's from our modelization.
            ctx.registry.borrow_mut().create_type(
                |registry| MetaType::Object {
                    name: type_name.clone(),
                    description: type_definition.node.description.clone().map(|x| x.node),
                    fields: {
                        let mut fields = IndexMap::new();
                        for field in &object.fields {
                            let name = field.node.name.node.to_string();

                            // Will be added later.
                            if RESERVED_FIELDS.contains(&name.as_str()) {
                                continue;
                            }

                            let (resolver, relation, transformer, edges, args, ty) =
                                ResolverDirective::resolver_name(&field.node)
                                    .map(|resolver_name| {
                                        (
                                            Resolver {
                                                id: Some(format!("{}_custom_resolver", type_name.to_lowercase())),
                                                r#type: ResolverType::CustomResolver(CustomResolver {
                                                    resolver_name: resolver_name.to_owned(),
                                                }),
                                            },
                                            None,
                                            None,
                                            vec![],
                                            Default::default(),
                                            field.node.ty.clone().node.to_string(),
                                        )
                                    })
                                    .or_else(|| {
                                        RelationEngine::get(ctx, &type_definition.node.name.node, &field.node).map(
                                            |relation| {
                                                let id = Some(format!("{}_edge_resolver", type_name.to_lowercase()));
                                                let edges = {
                                                    let edge_type = to_base_type_str(&field.node.ty.node.base);
                                                    connection_edges.push(edge_type.clone());
                                                    vec![edge_type]
                                                };
                                                let (context_data_resolver, args, ty) =
                                                    if is_array_basic_type(&field.node.ty.to_string()) {
                                                        (
                                                            ContextDataResolver::EdgeArray {
                                                                key: type_name.clone(),
                                                                relation_name: relation.name.clone(),
                                                                expected_ty: to_base_type_str(&field.node.ty.node.base),
                                                            },
                                                            generate_pagination_args(registry, &type_definition.node),
                                                            format!(
                                                                "{}Connection",
                                                                to_base_type_str(&field.node.ty.node.base).to_camel()
                                                            ),
                                                        )
                                                    } else {
                                                        (
                                                            ContextDataResolver::SingleEdge {
                                                                key: type_name.clone(),
                                                                relation_name: relation.name.clone(),
                                                            },
                                                            Default::default(),
                                                            field.node.ty.clone().node.to_string(),
                                                        )
                                                    };
                                                (
                                                    Resolver {
                                                        id,
                                                        r#type: ResolverType::ContextDataResolver(
                                                            context_data_resolver,
                                                        ),
                                                    },
                                                    Some(relation),
                                                    None,
                                                    edges,
                                                    args,
                                                    ty,
                                                )
                                            },
                                        )
                                    })
                                    .unwrap_or_else(|| {
                                        (
                                            Resolver {
                                                id: None,
                                                r#type: ResolverType::ContextDataResolver(
                                                    ContextDataResolver::LocalKey {
                                                        key: type_name.to_string(),
                                                    },
                                                ),
                                            },
                                            None,
                                            Some(Transformer::DynamoSelect { property: name.clone() }),
                                            vec![],
                                            Default::default(),
                                            field.node.ty.clone().node.to_string(),
                                        )
                                    });

                            let plan = match &relation {
                                None => Some(SchemaPlan::projection(vec![name.clone()])),
                                Some(meta_relation) => Some(SchemaPlan::related(
                                    Some(ctx.get_schema_id(&meta_relation.relation.0.clone().unwrap())),
                                    ctx.get_schema_id(&meta_relation.relation.1.clone()),
                                    Some(meta_relation.name.clone()),
                                )),
                            };

                            fields.insert(
                                name.clone(),
                                MetaField {
                                    name: name.clone(),
                                    description: field.node.description.clone().map(|x| x.node),
                                    args,
                                    ty,
                                    deprecation: Default::default(),
                                    cache_control: Default::default(),
                                    external: false,
                                    requires: None,
                                    provides: None,
                                    visible: None,
                                    compute_complexity: None,
                                    resolve: Some(resolver),
                                    edges,
                                    plan,
                                    relation,
                                    transformer,
                                    required_operation: None,
                                    auth: field_auth.get(&name).expect("must be set").clone(),
                                },
                            );
                        }
                        insert_metadata_field(
                            &mut fields,
                            &type_name,
                            RESERVED_FIELD_ID,
                            Some("Unique identifier".to_owned()),
                            "ID!",
                            dynamodb::constant::SK,
                            "id",
                            field_auth
                                .get(RESERVED_FIELD_ID)
                                .map(|e| e.as_ref())
                                .unwrap_or(model_auth.as_ref()),
                        );
                        insert_metadata_field(
                            &mut fields,
                            &type_name,
                            RESERVED_FIELD_UPDATED_AT,
                            Some("when the model was updated".to_owned()),
                            "DateTime!",
                            dynamodb::constant::UPDATED_AT,
                            "updatedAt",
                            field_auth
                                .get(RESERVED_FIELD_UPDATED_AT)
                                .map(|e| e.as_ref())
                                .unwrap_or(model_auth.as_ref()),
                        );
                        insert_metadata_field(
                            &mut fields,
                            &type_name,
                            RESERVED_FIELD_CREATED_AT,
                            Some("when the model was created".to_owned()),
                            "DateTime!",
                            dynamodb::constant::CREATED_AT,
                            "createdAt",
                            field_auth
                                .get(RESERVED_FIELD_CREATED_AT)
                                .map(|e| e.as_ref())
                                .unwrap_or(model_auth.as_ref()),
                        );

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
                    constraints: unique_directives.iter().map(UniqueDirective::to_constraint).collect(),
                },
                &type_name,
                &type_name,
            );

            //
            // GENERATE QUERY ONE OF: type(by: { ... })
            //

            let one_of_type_name = format!("{type_name}ByInput");
            ctx.registry.get_mut().create_type(
                |registry| MetaType::InputObject {
                    name: one_of_type_name.clone(),
                    description: type_definition
                        .node
                        .description
                        .clone()
                        .map(|description| description.node),
                    visible: None,
                    rust_typename: one_of_type_name.clone(),
                    input_fields: {
                        let mut input_fields = IndexMap::new();
                        input_fields.insert(
                            RESERVED_FIELD_ID.to_string(),
                            MetaInputValue {
                                name: RESERVED_FIELD_ID.to_string(),
                                description: None,
                                ty: "ID".to_string(),
                                default_value: None,
                                validators: None,
                                visible: None,
                                is_secret: false,
                            },
                        );
                        for unique_directive in &unique_directives {
                            input_fields.insert(unique_directive.name(), unique_directive.lookup_by_field(registry));
                        }
                        input_fields
                    },
                    oneof: true,
                },
                &one_of_type_name,
                &one_of_type_name,
            );

            ctx.queries.push(MetaField {
                // "by" query
                name: to_lower_camelcase(&type_name),
                description: Some(format!("Query a single {type_name} by an ID or a unique field")),
                args: {
                    let mut args = IndexMap::new();
                    args.insert(
                        "by".to_owned(),
                        MetaInputValue {
                            name: "by".to_owned(),
                            ty: format!("{one_of_type_name}!"),
                            description: Some(format!("The field and value by which to query the {type_name}")),
                            validators: None,
                            visible: None,
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
                        schema: Some(schema_id),
                    }),
                }),
                plan: None,
                transformer: None,
                required_operation: Some(Operations::GET),
                auth: model_auth.clone(),
            });

            //
            // ADD FURTHER QUERIES/MUTATIONS
            //
            add_mutation_create(ctx, &type_definition.node, object, model_auth.as_ref());
            add_mutation_update(ctx, &type_definition.node, object, model_auth.as_ref());

            add_query_paginated_collection(ctx, &type_definition.node, connection_edges, model_auth.as_ref());
            add_remove_mutation(ctx, &type_name, model_auth.as_ref());

            let search_fields = object
                .fields
                .iter()
                .filter_map(|field| {
                    field
                        .node
                        .directives
                        .iter()
                        .find(|directive| directive.node.name.node == SEARCH_DIRECTIVE)
                        .map(|directive| (&field.node, &directive.node))
                })
                .collect::<Vec<_>>();
            if !search_fields.is_empty() {
                add_query_search(ctx, &type_definition.node, model_auth.as_ref(), search_fields);
            }
        }
    }
}

fn has_any_invalid_reserved_fields(ctx: &mut VisitorContext<'_>, object_name: &str, object: &ObjectType) -> bool {
    let mut has_invalid_field = false;
    for field in &object.fields {
        let field_name = field.node.name.node.as_str();
        let expected_type_name = match field_name {
            RESERVED_FIELD_CREATED_AT | RESERVED_FIELD_UPDATED_AT => DateTimeScalar::name(),
            RESERVED_FIELD_ID => IDScalar::name(),
            // Field is not reserved.
            _ => continue,
        }
        .expect("Reserved field with an unnamed Scalar cannot happen.");

        if_chain! {
            if let BaseType::Named(type_name) = &field.node.ty.node.base;
            // reserved fields are supposed to be always required.
            if type_name == expected_type_name && !field.node.ty.node.nullable;
            then {}
            else {
                has_invalid_field = true;
                ctx.report_error(
                    vec![field.pos],
                    format!("Field '{field_name}' of '{object_name}' is reserved by @model directive. It must have the type '{expected_type_name}!' if present."),
                );
            }
        }
    }
    has_invalid_field
}

#[cfg(test)]
mod tests {
    use serde_json as _;

    use dynaql::{AuthConfig, Operations};
    use dynaql_parser::parse_schema;
    use std::collections::HashMap;

    use crate::rules::visitor::{visit, VisitorContext};

    use super::ModelDirective;

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

    #[test]
    fn should_handle_model_auth() {
        let schema = r#"
            type Todo @model @auth(rules: [ { allow: private } ]) {
                id: ID!
                title: String
            }
            "#;

        let variables = HashMap::new();
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_with_variables(&schema, &variables);
        visit(&mut ModelDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "errors: {:?}", ctx.errors);

        let expected_model_auth = AuthConfig {
            allowed_private_ops: Operations::all(),
            ..Default::default()
        };

        let tests = vec![
            ("TodoCreatePayload", "todo", Some(Operations::CREATE)),
            ("TodoUpdatePayload", "todo", Some(Operations::UPDATE)),
            ("TodoDeletePayload", "deletedId", Some(Operations::DELETE)),
            ("PageInfo", "hasPreviousPage", Some(Operations::LIST)),
            ("PageInfo", "hasNextPage", Some(Operations::LIST)),
            ("PageInfo", "startCursor", Some(Operations::LIST)),
            ("PageInfo", "endCursor", Some(Operations::LIST)),
            ("TodoConnection", "pageInfo", Some(Operations::LIST)),
            ("TodoConnection", "edges", Some(Operations::LIST)),
            ("TodoEdge", "node", Some(Operations::LIST)),
            ("TodoEdge", "cursor", Some(Operations::LIST)),
            ("Todo", "id", None),
            ("Todo", "title", None),
            ("Todo", "createdAt", None),
            ("Todo", "updatedAt", None),
        ];

        let types = &ctx.registry.borrow().types;

        for (type_name, field_name, required_op) in tests {
            let field = types[type_name].field_by_name(field_name).unwrap();
            assert_eq!(
                field.auth.as_ref(),
                // PageInfo is not specific to the model. The model_auth should be passed down
                // during resolution.
                if type_name == "PageInfo" {
                    None
                } else {
                    Some(&expected_model_auth)
                },
                "{type_name}.{field_name}"
            );
            assert_eq!(field.required_operation, required_op, "{type_name}.{field_name}");
        }
    }

    #[test]
    fn should_handle_field_auth() {
        let schema = r#"
            type Todo @model {
                id: ID!
                title: String @auth(rules: [{ allow: owner }])
            }
            "#;

        let variables = HashMap::new();
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_with_variables(&schema, &variables);
        visit(&mut ModelDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "errors: {:?}", ctx.errors);

        let expected_field_auth = AuthConfig {
            allowed_owner_ops: Operations::all(),
            ..Default::default()
        };

        let tests = vec![
            ("Todo", "id", None, None),
            ("Todo", "title", Some(&expected_field_auth), None),
            ("Todo", "createdAt", None, None),
            ("Todo", "updatedAt", None, None),
        ];

        let types = &ctx.registry.borrow().types;

        for (type_name, field_name, auth, required_op) in tests {
            let field = types[type_name].field_by_name(field_name).unwrap();
            assert_eq!(field.auth.as_ref(), auth, "{type_name}.{field_name}");
            assert_eq!(field.required_operation, required_op, "{type_name}.{field_name}");
        }
    }

    #[test]
    fn should_handle_model_and_field_auth() {
        let schema = r#"
            type Todo @model @auth(rules: [ { allow: private } ]) {
                id: ID!
                title: String @auth(rules: [{ allow: owner }])
            }
            "#;

        let variables = HashMap::new();
        let schema = parse_schema(schema).unwrap();
        let mut ctx = VisitorContext::new_with_variables(&schema, &variables);
        visit(&mut ModelDirective, &mut ctx, &schema);

        assert!(ctx.errors.is_empty(), "errors: {:?}", ctx.errors);

        let expected_model_auth = AuthConfig {
            allowed_private_ops: Operations::all(),
            ..Default::default()
        };
        let expected_field_auth = AuthConfig {
            allowed_owner_ops: Operations::all(),
            ..Default::default()
        };

        let tests = vec![
            ("Todo", "id", Some(&expected_model_auth), None),
            ("Todo", "title", Some(&expected_field_auth), None),
            ("Todo", "createdAt", Some(&expected_model_auth), None),
            ("Todo", "updatedAt", Some(&expected_model_auth), None),
        ];

        let types = &ctx.registry.borrow().types;

        for (type_name, field_name, auth, required_op) in tests {
            let field = types[type_name].field_by_name(field_name).unwrap();
            assert_eq!(field.auth.as_ref(), auth, "{type_name}.{field_name}");
            assert_eq!(field.required_operation, required_op, "{type_name}.{field_name}");
        }
    }
}
