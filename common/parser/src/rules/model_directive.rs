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
use dynaql::names::{INPUT_FIELD_FILTER_ALL, INPUT_FIELD_FILTER_ANY, INPUT_FIELD_FILTER_NONE, INPUT_FIELD_FILTER_NOT};
use dynaql::registry::resolvers::custom::CustomResolver;
use if_chain::if_chain;

use dynaql::indexmap::IndexMap;
use dynaql::registry::resolvers::dynamo_querying::DynamoResolver;
use dynaql::registry::scalars::{DateTimeScalar, IDScalar, SDLDefinitionScalar};
use dynaql::registry::{self, MetaField};
use dynaql::registry::{is_array_basic_type, MetaType};
use dynaql::registry::{
    resolvers::transformer::Transformer, resolvers::Resolver, variables::VariableResolveDefinition,
};
use dynaql::registry::{InputObjectType, MetaInputValue};
use dynaql::{AuthConfig, Positioned};
use dynaql_parser::types::{BaseType, FieldDefinition, ObjectType, Type, TypeDefinition, TypeKind};
use grafbase::auth::Operations;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::registry::generate_pagination_args;
use crate::registry::names::MetaNames;
use crate::registry::{
    add_mutation_create, add_mutation_delete, add_mutation_update, add_query_paginated_collection, add_query_search,
};
use crate::rules::cache_directive::CacheDirective;
use crate::utils::to_base_type_str;
use crate::utils::to_lower_camelcase;

use super::auth_directive::AuthDirective;
use super::directive::Directive;
use super::relations::RelationEngine;
use super::resolver_directive::ResolverDirective;
use super::unique_directive::UniqueDirective;
use super::visitor::{Visitor, VisitorContext};

pub struct ModelDirective;

pub const METADATA_FIELD_CREATED_AT: &str = "createdAt";
pub const METADATA_FIELD_UPDATED_AT: &str = "updatedAt";
pub const METADATA_FIELDS: [&str; 3] = [
    dynaql::names::OUTPUT_FIELD_ID,
    METADATA_FIELD_UPDATED_AT,
    METADATA_FIELD_CREATED_AT,
];
pub const RESERVED_FIELDS: [&str; 4] = [
    INPUT_FIELD_FILTER_ALL,
    INPUT_FIELD_FILTER_ANY,
    INPUT_FIELD_FILTER_NONE,
    INPUT_FIELD_FILTER_NOT,
];
pub const MODEL_DIRECTIVE: &str = "model";

impl ModelDirective {
    pub fn is_not_metadata_field(field: &Positioned<FieldDefinition>) -> bool {
        !METADATA_FIELDS.contains(&field.node.name.node.as_str())
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
                    if ty.node.directives.iter().any(|directive| {
                        let is_model = directive.node.name.node == MODEL_DIRECTIVE;
                        let has_no_attributes = directive.node.arguments.is_empty();

                        is_model && has_no_attributes
                    });
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
    auth: Option<&AuthConfig>,
) -> Option<MetaField> {
    fields.insert(
        field_name.to_owned(),
        MetaField {
            name: field_name.to_owned(),
            mapped_name: None,
            description,
            args: Default::default(),
            ty: ty.into(),
            deprecation: Default::default(),
            cache_control: Default::default(),
            external: false,
            requires: None,
            provides: None,
            visible: None,
            compute_complexity: None,
            resolver: Transformer::select(type_name).and_then(Transformer::DynamoSelect {
                key: dynamo_property_name.to_owned(),
            }),
            edges: Vec::new(),
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
            .filter(|directive| directive.node.name.node == MODEL_DIRECTIVE)
            .any(|directive| directive.node.arguments.is_empty())
        {
            return;
        }
        if let TypeKind::Object(object) = &type_definition.node.kind {
            let type_name = MetaNames::model(&type_definition.node);
            if type_definition.node.name.node != type_name {
                ctx.report_error(
                    vec![type_definition.node.name.pos],
                    format!(
                        "Models must be named in PascalCase.  Try renaming {} to {type_name}.",
                        type_definition.node.name.node
                    ),
                );
                return;
            }
            if has_any_invalid_metadata_fields(ctx, &type_name, object) {
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
            let schema_id = ctx.get_schema_id(&type_name);

            for field in &object.fields {
                let name = field.node.name.node.to_string();
                if RESERVED_FIELDS.contains(&name.as_str()) {
                    ctx.report_error(
                        vec![field.pos],
                        format!("Field name '{name}' is reserved and cannot be used."),
                    );
                }
            }

            let model_cache = CacheDirective::parse(&type_definition.node.directives);

            //
            // CREATE ACTUAL TYPE
            //
            let mut connection_edges = Vec::new();
            // If it's a modeled Type, we create the associated type into the registry.
            // Without more data, we infer it's from our modelization.
            ctx.registry.borrow_mut().create_type(
                |registry| {
                    let mut fields = IndexMap::new();
                    for field in &object.fields {
                        let name = field.node.name.node.to_string();

                        // Will be added later or ignored (error was already reported)
                        if METADATA_FIELDS.contains(&name.as_str()) || RESERVED_FIELDS.contains(&name.as_str()) {
                            continue;
                        }

                        let (resolver, relation, edges, args, ty, cache_control) =
                            ResolverDirective::resolver_name(&field.node)
                                .map(|resolver_name| {
                                    (
                                        Resolver::CustomResolver(CustomResolver {
                                            resolver_name: resolver_name.to_owned(),
                                        }),
                                        None,
                                        vec![],
                                        field
                                            .node
                                            .arguments
                                            .iter()
                                            .map(|argument| {
                                                (
                                                    argument.node.name.to_string(),
                                                    MetaInputValue::new(
                                                        argument.node.name.to_string(),
                                                        argument.node.ty.to_string(),
                                                    ),
                                                )
                                            })
                                            .collect(),
                                        field.node.ty.clone().node.to_string(),
                                        CacheDirective::parse(&field.node.directives),
                                    )
                                })
                                .or_else(|| {
                                    RelationEngine::get(ctx, &type_name, &field.node).map(|relation| {
                                        let edges = {
                                            let edge_type = to_base_type_str(&field.node.ty.node.base);
                                            connection_edges.push(edge_type.clone());
                                            vec![edge_type]
                                        };
                                        let (context_data_resolver, args, ty) =
                                            if is_array_basic_type(&field.node.ty.to_string()) {
                                                (
                                                    Transformer::EdgeArray {
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
                                                    Transformer::SingleEdge {
                                                        key: type_name.clone(),
                                                        relation_name: relation.name.clone(),
                                                    },
                                                    Default::default(),
                                                    field.node.ty.clone().node.to_string(),
                                                )
                                            };
                                        (
                                            Resolver::Transformer(context_data_resolver),
                                            Some(relation),
                                            edges,
                                            args,
                                            ty,
                                            CacheDirective::parse(&field.node.directives),
                                        )
                                    })
                                })
                                .unwrap_or_else(|| {
                                    (
                                        Resolver::Transformer(Transformer::Select {
                                            key: type_name.to_string(),
                                        })
                                        .and_then(Transformer::DynamoSelect { key: name.clone() }),
                                        None,
                                        vec![],
                                        Default::default(),
                                        field.node.ty.clone().node.to_string(),
                                        CacheDirective::parse(&field.node.directives),
                                    )
                                });

                        fields.insert(
                            name.clone(),
                            MetaField {
                                auth: field_auth.get(&name).expect("must be set").clone(),
                                name,
                                description: field.node.description.clone().map(|x| x.node),
                                args,
                                ty: ty.into(),
                                cache_control,
                                resolver,
                                edges,
                                relation,
                                required_operation: None,
                                ..Default::default()
                            },
                        );
                    }
                    insert_metadata_field(
                        &mut fields,
                        &type_name,
                        dynaql::names::OUTPUT_FIELD_ID,
                        Some("Unique identifier".to_owned()),
                        "ID!",
                        dynamodb::constant::SK,
                        field_auth
                            .get(dynaql::names::OUTPUT_FIELD_ID)
                            .map(|e| e.as_ref())
                            .unwrap_or(model_auth.as_ref()),
                    );
                    insert_metadata_field(
                        &mut fields,
                        &type_name,
                        METADATA_FIELD_UPDATED_AT,
                        Some("when the model was updated".to_owned()),
                        "DateTime!",
                        dynamodb::constant::UPDATED_AT,
                        field_auth
                            .get(METADATA_FIELD_UPDATED_AT)
                            .map(|e| e.as_ref())
                            .unwrap_or(model_auth.as_ref()),
                    );
                    insert_metadata_field(
                        &mut fields,
                        &type_name,
                        METADATA_FIELD_CREATED_AT,
                        Some("when the model was created".to_owned()),
                        "DateTime!",
                        dynamodb::constant::CREATED_AT,
                        field_auth
                            .get(METADATA_FIELD_CREATED_AT)
                            .map(|e| e.as_ref())
                            .unwrap_or(model_auth.as_ref()),
                    );

                    MetaType::Object(registry::ObjectType {
                        name: type_name.clone(),
                        description: type_definition.node.description.clone().map(|x| x.node),
                        fields,
                        cache_control: model_cache.clone(),
                        extends: false,
                        keys: None,
                        visible: None,
                        is_subscription: false,
                        is_node: true,
                        rust_typename: type_name.clone(),
                        constraints: unique_directives.iter().map(UniqueDirective::to_constraint).collect(),
                    })
                },
                &type_name,
                &type_name,
            );

            //
            // GENERATE QUERY ONE OF: type(by: { ... })
            //

            let one_of_type_name = MetaNames::by_input(&type_definition.node);
            ctx.registry.get_mut().create_type(
                |registry| {
                    let mut input_fields = vec![];
                    input_fields.push(MetaInputValue::new(dynaql::names::OUTPUT_FIELD_ID, "ID".to_string()));
                    for unique_directive in &unique_directives {
                        input_fields.push(unique_directive.lookup_by_field(registry));
                    }
                    InputObjectType::new(one_of_type_name.clone(), input_fields)
                        .with_description(
                            type_definition
                                .node
                                .description
                                .clone()
                                .map(|description| description.node),
                        )
                        .with_oneof(true)
                        .into()
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
                        MetaInputValue::new("by", format!("{one_of_type_name}!"))
                            .with_description(format!("The field and value by which to query the {type_name}")),
                    );
                    args
                },
                ty: type_name.clone().into(),
                deprecation: dynaql::registry::Deprecation::NoDeprecated,
                cache_control: model_cache.clone(),
                // TODO: Should be defined as a ResolveNode
                // Single entity
                resolver: Resolver::DynamoResolver(DynamoResolver::QueryBy {
                    by: VariableResolveDefinition::InputTypeName("by".to_owned()),
                    schema: Some(schema_id),
                }),
                required_operation: Some(Operations::GET),
                auth: model_auth.clone(),
                ..Default::default()
            });

            //
            // ADD FURTHER QUERIES/MUTATIONS
            //
            add_mutation_create(ctx, &type_definition.node, object, model_auth.as_ref());
            add_mutation_update(ctx, &type_definition.node, object, model_auth.as_ref());
            add_mutation_delete(ctx, &type_definition.node, model_auth.as_ref(), model_cache);

            add_query_paginated_collection(ctx, &type_definition.node, connection_edges, model_auth.as_ref());
            add_query_search(ctx, &type_definition.node, &object.fields, model_auth.as_ref());
        }
    }
}

fn has_any_invalid_metadata_fields(ctx: &mut VisitorContext<'_>, object_name: &str, object: &ObjectType) -> bool {
    let mut has_invalid_field = false;
    for field in &object.fields {
        let field_name = field.node.name.node.as_str();
        let expected_type_name = match field_name {
            METADATA_FIELD_CREATED_AT | METADATA_FIELD_UPDATED_AT => DateTimeScalar::name(),
            dynaql::names::OUTPUT_FIELD_ID => IDScalar::name(),
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

    use dynaql::AuthConfig;
    use dynaql_parser::parse_schema;
    use grafbase::auth::Operations;
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
