use std::collections::HashMap;

use dynaql::registry::resolvers::debug::DebugResolver;
use itertools::Itertools;

use dynaql::names::INPUT_FIELD_FILTER_IS_NULL;
use dynaql::registry::resolvers::context_data::ContextDataResolver;
use dynaql::registry::resolvers::dynamo_querying::DynamoResolver;
use dynaql::registry::resolvers::query::{QueryResolver, SEARCH_RESOLVER_HIT_IDS, SEARCH_RESOLVER_TOTAL_HITS};
use dynaql::registry::{
    resolvers::Resolver, resolvers::ResolverType, variables::VariableResolveDefinition, MetaField, MetaInputValue,
};
use dynaql::registry::{MetaType, MetaTypeName, Registry};
use dynaql::{AuthConfig, Operations, Positioned};
use dynaql_parser::types::{ConstDirective, FieldDefinition, Type, TypeDefinition};
use grafbase_runtime::search;

use crate::registry::generate_pagination_args;
use crate::registry::names::{
    MetaNames, INPUT_ARG_FIELDS, INPUT_ARG_FILTER, INPUT_ARG_QUERY, INPUT_FIELD_FILTER_ALL, INPUT_FIELD_FILTER_ANY,
    INPUT_FIELD_FILTER_EQ, INPUT_FIELD_FILTER_GT, INPUT_FIELD_FILTER_GTE, INPUT_FIELD_FILTER_IN, INPUT_FIELD_FILTER_LT,
    INPUT_FIELD_FILTER_LTE, INPUT_FIELD_FILTER_NEQ, INPUT_FIELD_FILTER_NOT, INPUT_FIELD_FILTER_NOT_IN,
    PAGINATION_FIELD_EDGES, PAGINATION_FIELD_EDGE_CURSOR, PAGINATION_FIELD_EDGE_NODE,
    PAGINATION_FIELD_EDGE_SEARCH_SCORE, PAGINATION_FIELD_PAGE_INFO, PAGINATION_FIELD_SEARCH_INFO,
    PAGINATION_INPUT_ARG_AFTER, PAGINATION_INPUT_ARG_BEFORE, PAGINATION_INPUT_ARG_FIRST, PAGINATION_INPUT_ARG_LAST,
    SEARCH_INFO_FIELD_TOTAL_HITS, SEARCH_INFO_TYPE,
};
use crate::rules::search_directive::SEARCH_DIRECTIVE;
use crate::rules::visitor::VisitorContext;

fn convert_to_search_field_type(ty: &str, is_nullable: Option<bool>) -> Result<search::FieldType, String> {
    match MetaTypeName::create(ty) {
        MetaTypeName::NonNull(type_name) => convert_to_search_field_type(type_name, is_nullable.or(Some(false))),
        MetaTypeName::List(type_name) => convert_to_search_field_type(type_name, Some(true)),
        MetaTypeName::Named(type_name) => {
            let opts = search::FieldOptions {
                nullable: is_nullable.unwrap_or(true),
            };
            Ok(match type_name {
                "URL" => search::FieldType::URL(opts),
                "Email" => search::FieldType::Email(opts),
                "PhoneNumber" => search::FieldType::PhoneNumber(opts),
                "String" => search::FieldType::String(opts),
                "Date" => search::FieldType::Date(opts),
                "DateTime" => search::FieldType::DateTime(opts),
                "Timestamp" => search::FieldType::Timestamp(opts),
                "Int" => search::FieldType::Int(opts),
                "Float" => search::FieldType::Float(opts),
                "Boolean" => search::FieldType::Boolean(opts),
                "IPAddress" => search::FieldType::IPAddress(opts),
                _ => return Err(type_name.to_string()),
            })
        }
    }
}

pub fn add_query_search(
    ctx: &mut VisitorContext<'_>,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&AuthConfig>,
    search_fields: Vec<(&FieldDefinition, &Positioned<ConstDirective>)>,
) {
    assert!(!search_fields.is_empty());
    let type_name = MetaNames::model(model_type_definition);
    // FIXME: At several places the lowercase for the id & entity_type is
    // used. A single code path should handle that.
    let entity_type = type_name.to_lowercase();
    let field_name = MetaNames::query_search(model_type_definition);

    let (fields, errors): (HashMap<_, _>, Vec<_>) = search_fields
        .into_iter()
        .map(|(field, directive)| {
            convert_to_search_field_type(&field.ty.node.to_string(), None)
                .map(|ty| (field.name.node.to_string(), search::FieldEntry { ty }))
                .map_err(|unsupported_type_name| {
                    ctx.report_error(
                        vec![directive.pos],
                        format!(
                            "The @{SEARCH_DIRECTIVE} directive cannot be used with the {unsupported_type_name} type."
                        ),
                    );
                })
        })
        .partition_result();
    if !errors.is_empty() {
        return;
    }
    let config = search::IndexConfig {
        schema: search::Schema { fields },
    };

    let connection_type = register_connection_type(ctx.registry.get_mut(), model_type_definition, model_auth);
    ctx.queries.push(MetaField {
        name: field_name,
        description: Some(format!("Search `{type_name}`")),
        args: {
            let mut pagination_args = generate_pagination_args(ctx.registry.get_mut(), model_type_definition);
            let args = vec![
                MetaInputValue::new(INPUT_ARG_QUERY, "String").with_description("Text to search."),
                MetaInputValue::new(INPUT_ARG_FIELDS, "[String!]").with_description(concat!(
                    "Fields used for searching. ",
                    "Restricted to String, URL, Email and PhoneNumber fields. ",
                    "If not specified it defaults to all @search fields with those types."
                )),
                MetaInputValue::new(
                    INPUT_ARG_FILTER,
                    register_model_filter(ctx.registry.get_mut(), model_type_definition, &config.schema),
                ),
                pagination_args
                    .remove(PAGINATION_INPUT_ARG_FIRST)
                    .expect("Has to be present"),
                pagination_args
                    .remove(PAGINATION_INPUT_ARG_AFTER)
                    .expect("Has to be present"),
                pagination_args
                    .remove(PAGINATION_INPUT_ARG_LAST)
                    .expect("Has to be present"),
                pagination_args
                    .remove(PAGINATION_INPUT_ARG_BEFORE)
                    .expect("Has to be present"),
            ];

            args.into_iter().map(|input| (input.name.clone(), input)).collect()
        },
        ty: connection_type,
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
        edges: vec![],
        relation: None,
        plan: None,
        resolve: Some(Resolver {
            id: None,
            r#type: ResolverType::Query(QueryResolver::Search {
                query: VariableResolveDefinition::InputTypeName(INPUT_ARG_QUERY.to_string()),
                fields: VariableResolveDefinition::InputTypeName(INPUT_ARG_FIELDS.to_string()),
                filter: VariableResolveDefinition::InputTypeName(INPUT_ARG_FILTER.to_string()),
                first: VariableResolveDefinition::InputTypeName(PAGINATION_INPUT_ARG_FIRST.to_string()),
                after: VariableResolveDefinition::InputTypeName(PAGINATION_INPUT_ARG_AFTER.to_string()),
                before: VariableResolveDefinition::InputTypeName(PAGINATION_INPUT_ARG_BEFORE.to_string()),
                last: VariableResolveDefinition::InputTypeName(PAGINATION_INPUT_ARG_LAST.to_string()),
                entity_type: entity_type.clone(),
            }),
        }),
        transformer: None,
        required_operation: Some(Operations::LIST),
        auth: model_auth.cloned(),
    });

    ctx.registry.get_mut().search_config.indices.insert(entity_type, config);
}

fn register_connection_type(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&AuthConfig>,
) -> String {
    let type_name = MetaNames::search_connection_type(model_type_definition);

    registry.create_type(
        |registry| {
            let edge_type = register_edge_type(registry, model_type_definition, model_auth);
            let search_info_type = register_search_info(registry);
            let page_info_type = Type::required(super::pagination::register_page_info_type(registry)).to_string();
            MetaType::Object {
                name: type_name.clone(),
                description: None,
                fields: vec![
                    MetaField {
                        name: PAGINATION_FIELD_PAGE_INFO.to_string(),
                        ty: page_info_type,
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGINATION_FIELD_SEARCH_INFO.to_string(),
                        ty: search_info_type,
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGINATION_FIELD_EDGES.to_string(),
                        ty: format!("[{edge_type}!]!"),
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        resolve: Some(Resolver {
                            id: None,
                            r#type: ResolverType::DynamoResolver(DynamoResolver::QueryIds {
                                ids: VariableResolveDefinition::LocalData(SEARCH_RESOLVER_HIT_IDS.to_string()),
                                type_name: MetaNames::model(model_type_definition),
                            }),
                        }),
                        ..Default::default()
                    },
                ]
                .into_iter()
                .map(|input| (input.name.clone(), input))
                .collect(),
                cache_control: Default::default(),
                extends: false,
                keys: None,
                visible: None,
                is_subscription: false,
                is_node: false,
                rust_typename: type_name.clone(),
                constraints: vec![],
            }
        },
        &type_name,
        &type_name,
    );

    type_name
}

fn register_search_info(registry: &mut Registry) -> String {
    let type_name = SEARCH_INFO_TYPE.to_string();
    registry.create_type(
        |_| MetaType::Object {
            name: type_name.clone(),
            description: None,
            fields: vec![MetaField {
                name: SEARCH_INFO_FIELD_TOTAL_HITS.to_string(),
                ty: "Int!".to_string(),
                resolve: Some(Resolver {
                    id: None,
                    r#type: ResolverType::ContextDataResolver(ContextDataResolver::LocalKey {
                        key: SEARCH_RESOLVER_TOTAL_HITS.to_string(),
                    }),
                }),
                ..Default::default()
            }]
            .into_iter()
            .map(|input| (input.name.clone(), input))
            .collect(),
            cache_control: Default::default(),
            extends: false,
            keys: None,
            visible: None,
            is_subscription: false,
            is_node: false,
            rust_typename: type_name.clone(),
            constraints: vec![],
        },
        &type_name,
        &type_name,
    );

    type_name
}

fn register_edge_type(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&AuthConfig>,
) -> String {
    let type_name = MetaNames::search_edge_type(model_type_definition);
    registry.create_type(
        |_| MetaType::Object {
            name: type_name.clone(),
            description: None,
            fields: vec![
                MetaField {
                    name: PAGINATION_FIELD_EDGE_NODE.to_string(),
                    ty: format!("{}!", MetaNames::model(model_type_definition)),
                    required_operation: Some(Operations::LIST),
                    auth: model_auth.cloned(),
                    ..Default::default()
                },
                MetaField {
                    name: PAGINATION_FIELD_EDGE_CURSOR.to_string(),
                    ty: "String!".to_string(),
                    required_operation: Some(Operations::LIST),
                    auth: model_auth.cloned(),
                    resolve: Some(Resolver {
                        id: None,
                        r#type: ResolverType::DebugResolver(DebugResolver::Value {
                            inner: serde_json::Value::String(String::new()),
                        }),
                    }),
                    ..Default::default()
                },
                MetaField {
                    name: PAGINATION_FIELD_EDGE_SEARCH_SCORE.to_string(),
                    ty: "Float!".to_string(),
                    required_operation: Some(Operations::LIST),
                    auth: model_auth.cloned(),
                    resolve: Some(Resolver {
                        id: None,
                        r#type: ResolverType::DebugResolver(DebugResolver::Value {
                            inner: serde_json::json!(0.0),
                        }),
                    }),
                    ..Default::default()
                },
            ]
            .into_iter()
            .map(|input| (input.name.clone(), input))
            .collect(),
            cache_control: Default::default(),
            extends: false,
            keys: None,
            visible: None,
            is_subscription: false,
            is_node: false,
            rust_typename: type_name.clone(),
            constraints: vec![],
        },
        &type_name,
        &type_name,
    );

    type_name
}

fn register_model_filter(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    schema: &search::Schema,
) -> String {
    let input_type_name = MetaNames::search_filter_input(model_type_definition);
    registry.create_type(
        |registry| MetaType::InputObject {
            name: input_type_name.clone(),
            description: Some(String::new()),
            input_fields: {
                let mut args = schema
                    .fields
                    .iter()
                    .map(|(name, field)| MetaInputValue::new(name, register_scalar_filter(registry, &field.ty)))
                    .collect::<Vec<_>>();
                // Stable schema
                args.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

                args.extend([
                    MetaInputValue::new(INPUT_FIELD_FILTER_ALL, format!("[{input_type_name}!]")),
                    MetaInputValue::new(INPUT_FIELD_FILTER_ANY, format!("[{input_type_name}!]")),
                    MetaInputValue::new(INPUT_FIELD_FILTER_NOT, &input_type_name),
                ]);

                args.into_iter().map(|input| (input.name.clone(), input)).collect()
            },
            visible: None,
            rust_typename: input_type_name.clone(),
            oneof: false,
        },
        &input_type_name,
        &input_type_name,
    );

    input_type_name
}

fn register_scalar_filter(registry: &mut Registry, ty: &search::FieldType) -> String {
    let scalar = ty.scalar_name();
    let input_type_name = MetaNames::search_scalar_filter_input(scalar, ty.is_nullable());
    registry.create_type(
        |_| MetaType::InputObject {
            name: input_type_name.clone(),
            description: Some(String::new()),
            input_fields: {
                let mut args = vec![
                    MetaInputValue::new(INPUT_FIELD_FILTER_EQ, scalar),
                    MetaInputValue::new(INPUT_FIELD_FILTER_NEQ, scalar),
                ];
                if scalar != "Boolean" {
                    args.extend([
                        MetaInputValue::new(INPUT_FIELD_FILTER_GT, scalar),
                        MetaInputValue::new(INPUT_FIELD_FILTER_GTE, scalar),
                        MetaInputValue::new(INPUT_FIELD_FILTER_LTE, scalar),
                        MetaInputValue::new(INPUT_FIELD_FILTER_LT, scalar),
                        MetaInputValue::new(INPUT_FIELD_FILTER_IN, format!("[{scalar}!]")),
                        MetaInputValue::new(INPUT_FIELD_FILTER_NOT_IN, format!("[{scalar}!]")),
                    ]);
                }
                if ty.is_nullable() {
                    args.push(MetaInputValue::new(INPUT_FIELD_FILTER_IS_NULL, "Boolean"));
                }
                args.into_iter().map(|input| (input.name.clone(), input)).collect()
            },
            visible: None,
            rust_typename: input_type_name.clone(),
            oneof: scalar == "Boolean",
        },
        &input_type_name,
        &input_type_name,
    );

    input_type_name
}
