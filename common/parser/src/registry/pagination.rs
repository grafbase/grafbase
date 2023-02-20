use dynamodb::constant;
use dynaql::indexmap::IndexMap;
use dynaql::registry::enums::OrderByDirection;
use dynaql::registry::plan::SchemaPlan;
use dynaql::registry::relations::MetaRelation;
use dynaql::registry::transformers::Transformer;
use dynaql::registry::Registry;
use dynaql::registry::{
    resolvers::context_data::ContextDataResolver, resolvers::dynamo_querying::DynamoResolver, resolvers::Resolver,
    resolvers::ResolverType, variables::VariableResolveDefinition, MetaField, MetaInputValue, MetaType,
};
use dynaql::{AuthConfig, Operations};
use dynaql_parser::types::{BaseType, Type, TypeDefinition};

use crate::registry::names::{
    MetaNames, PAGE_INFO_FIELD_END_CURSOR, PAGE_INFO_FIELD_HAS_NEXT_PAGE, PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE,
    PAGE_INFO_FIELD_START_CURSOR, PAGE_INFO_TYPE, PAGINATION_FIELD_EDGES, PAGINATION_FIELD_EDGE_CURSOR,
    PAGINATION_FIELD_EDGE_NODE, PAGINATION_FIELD_PAGE_INFO,
};
use crate::rules::model_directive::RESERVED_FIELD_CREATED_AT;
use crate::rules::visitor::VisitorContext;

use super::names::{
    PAGINATION_INPUT_ARG_AFTER, PAGINATION_INPUT_ARG_BEFORE, PAGINATION_INPUT_ARG_FIRST, PAGINATION_INPUT_ARG_LAST,
    PAGINATION_INPUT_ARG_ORDER_BY,
};
use super::register_dynaql_enum;

fn register_edge_type(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&AuthConfig>,
) -> BaseType {
    let type_name = MetaNames::pagination_edge_type(model_type_definition);
    registry.create_type(
        |_| MetaType::Object {
            name: type_name.clone(),
            description: None,
            fields: IndexMap::from([
                (
                    PAGINATION_FIELD_EDGE_NODE.to_string(),
                    MetaField {
                        name: PAGINATION_FIELD_EDGE_NODE.to_string(),
                        description: None,
                        args: Default::default(),
                        ty: format!("{}!", MetaNames::model(model_type_definition)),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolve: None,
                        plan: None,
                        transformer: None,
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                    },
                ),
                (
                    PAGINATION_FIELD_EDGE_CURSOR.to_string(),
                    MetaField {
                        name: PAGINATION_FIELD_EDGE_CURSOR.to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "String!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolve: Some(Resolver {
                            id: None,
                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::LocalKey {
                                key: MetaNames::model(model_type_definition),
                            }),
                        }),
                        transformer: Some(Transformer::Pipeline(vec![
                            Transformer::DynamoSelect {
                                property: constant::SK.to_string(),
                            },
                            Transformer::ConvertSkToCursor,
                        ])),
                        // Incomplete: need base64
                        plan: Some(SchemaPlan::projection(vec!["id".to_string()])),
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                    },
                ),
            ]),
            cache_control: dynaql::CacheControl {
                public: true,
                max_age: 0usize,
            },
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
    BaseType::named(&type_name)
}

fn register_page_info_type(registry: &mut Registry) -> BaseType {
    registry.create_type(
        |_| MetaType::Object {
            name: PAGE_INFO_TYPE.to_string(),
            description: None,
            fields: IndexMap::from([
                (
                    PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE.to_string(),
                    MetaField {
                        name: PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE.to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "Boolean!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolve: Some(Resolver {
                            id: None,
                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::PaginationData),
                        }),
                        plan: None,
                        transformer: Some(Transformer::JSONSelect {
                            property: "has_previous_page".to_string(),
                        }),
                        required_operation: Some(Operations::LIST),
                        // TODO: Auth should be propagated down during resolution from the parent
                        // type. PageInfo type is not specific to any data model, what matters is
                        // the model authorization of the model on which we iterate over.
                        auth: None,
                    },
                ),
                (
                    PAGE_INFO_FIELD_HAS_NEXT_PAGE.to_string(),
                    MetaField {
                        name: PAGE_INFO_FIELD_HAS_NEXT_PAGE.to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "Boolean!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolve: Some(Resolver {
                            id: None,
                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::PaginationData),
                        }),
                        plan: None,
                        transformer: Some(Transformer::JSONSelect {
                            property: "has_next_page".to_string(),
                        }),
                        required_operation: Some(Operations::LIST),
                        auth: None,
                    },
                ),
                (
                    PAGE_INFO_FIELD_START_CURSOR.to_string(),
                    MetaField {
                        name: PAGE_INFO_FIELD_START_CURSOR.to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "String".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolve: Some(Resolver {
                            id: None,
                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::PaginationData),
                        }),
                        plan: None,
                        transformer: Some(Transformer::JSONSelect {
                            property: "start_cursor".to_string(),
                        }),
                        required_operation: Some(Operations::LIST),
                        auth: None,
                    },
                ),
                (
                    PAGE_INFO_FIELD_END_CURSOR.to_string(),
                    MetaField {
                        name: PAGE_INFO_FIELD_END_CURSOR.to_string(),
                        description: None,
                        args: Default::default(),
                        ty: "String".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        visible: None,
                        compute_complexity: None,
                        edges: Vec::new(),
                        relation: None,
                        resolve: Some(Resolver {
                            id: None,
                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::PaginationData),
                        }),
                        plan: None,
                        transformer: Some(Transformer::JSONSelect {
                            property: "end_cursor".to_string(),
                        }),
                        required_operation: Some(Operations::LIST),
                        auth: None,
                    },
                ),
            ]),
            cache_control: dynaql::CacheControl {
                public: true,
                max_age: 0usize,
            },
            extends: false,
            keys: None,
            visible: None,
            is_subscription: false,
            is_node: false,
            rust_typename: PAGE_INFO_TYPE.to_string(),
            constraints: vec![],
        },
        PAGE_INFO_TYPE,
        PAGE_INFO_TYPE,
    );
    BaseType::named(PAGE_INFO_TYPE)
}

fn register_connection_type(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    connection_edges: Vec<String>,
    model_auth: Option<&AuthConfig>,
) -> BaseType {
    let type_name = MetaNames::pagination_connection_type(model_type_definition);

    registry.create_type(
        |registry| {
            let edge_type = register_edge_type(registry, model_type_definition, model_auth);
            let page_info_type = register_page_info_type(registry);
            MetaType::Object {
                name: type_name.clone(),
                description: None,
                fields: IndexMap::from([
                    (
                        PAGINATION_FIELD_PAGE_INFO.to_string(),
                        MetaField {
                            name: PAGINATION_FIELD_PAGE_INFO.to_string(),
                            description: Some("Information to aid in pagination".to_string()),
                            args: Default::default(),
                            ty: Type::required(page_info_type).to_string(),
                            deprecation: Default::default(),
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                            visible: None,
                            compute_complexity: None,
                            edges: Vec::new(),
                            relation: None,
                            resolve: None,
                            plan: None,
                            transformer: None,
                            required_operation: Some(Operations::LIST),
                            auth: model_auth.cloned(),
                        },
                    ),
                    (
                        PAGINATION_FIELD_EDGES.to_string(),
                        MetaField {
                            name: PAGINATION_FIELD_EDGES.to_string(),
                            description: None,
                            args: Default::default(),
                            // TODO: Should this be really nullable?
                            ty: Type::nullable(BaseType::list(Type::nullable(edge_type))).to_string(),
                            deprecation: Default::default(),
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                            visible: None,
                            compute_complexity: None,
                            edges: connection_edges.clone(),
                            relation: None,
                            resolve: None,
                            plan: None,
                            transformer: None,
                            required_operation: Some(Operations::LIST),
                            auth: model_auth.cloned(),
                        },
                    ),
                ]),
                cache_control: dynaql::CacheControl {
                    public: true,
                    max_age: 0usize,
                },
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

    BaseType::named(&type_name)
}

/// Add a query to list a Collection with Relay specification and pagination
///
/// ```graphql
/// type Query {
///   entityCollection(first: Int, last: Int, after: String, before: String): EntityCollection
/// }
/// ```
pub fn add_query_paginated_collection(
    ctx: &mut VisitorContext<'_>,
    model_type_definition: &TypeDefinition,
    connection_edges: Vec<String>,
    model_auth: Option<&AuthConfig>,
) {
    let type_name = MetaNames::model(model_type_definition);
    let connection_type = register_connection_type(
        ctx.registry.get_mut(),
        model_type_definition,
        connection_edges,
        model_auth,
    );
    let field = MetaNames::query_collection(model_type_definition);

    let plan = Some(SchemaPlan::related(None, ctx.get_schema_id(&type_name), None));

    ctx.queries.push(MetaField {
        name: field.clone(),
        description: Some(format!("Paginated query to fetch the whole list of `{type_name}`.")),
        args: generate_pagination_args(ctx.registry.get_mut(), model_type_definition),
        // TODO: Should this be really nullable?
        ty: Type::nullable(connection_type).to_string(),
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
        relation: Some(MetaRelation::base_collection_relation(
            field,
            &Type::new(&type_name).expect("Shouldn't fail"),
        )),
        resolve: Some(Resolver {
            id: Some(format!("{}_resolver", type_name.to_lowercase())),
            // Multiple entities
            r#type: ResolverType::DynamoResolver(DynamoResolver::ListResultByTypePaginated {
                r#type: VariableResolveDefinition::DebugString(type_name.to_string()),
                first: VariableResolveDefinition::InputTypeName(PAGINATION_INPUT_ARG_FIRST.to_string()),
                after: VariableResolveDefinition::InputTypeName(PAGINATION_INPUT_ARG_AFTER.to_string()),
                before: VariableResolveDefinition::InputTypeName(PAGINATION_INPUT_ARG_BEFORE.to_string()),
                last: VariableResolveDefinition::InputTypeName(PAGINATION_INPUT_ARG_LAST.to_string()),
                order_by: Some(VariableResolveDefinition::InputTypeName(
                    PAGINATION_INPUT_ARG_ORDER_BY.to_string(),
                )),
                nested: None,
            }),
        }),
        plan,
        transformer: None,
        required_operation: Some(Operations::LIST),
        auth: model_auth.cloned(),
    });
}

pub fn generate_pagination_args(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
) -> IndexMap<String, MetaInputValue> {
    let orderby_input_type = register_orderby_input(registry, model_type_definition);
    IndexMap::from([
        (
            PAGINATION_INPUT_ARG_AFTER.to_string(),
            MetaInputValue {
                name: PAGINATION_INPUT_ARG_AFTER.to_string(),
                description: None,
                ty: "String".to_string(),
                default_value: None,
                validators: None,
                visible: None,
                is_secret: false,
            },
        ),
        (
            PAGINATION_INPUT_ARG_BEFORE.to_string(),
            MetaInputValue {
                name: PAGINATION_INPUT_ARG_BEFORE.to_string(),
                description: None,
                ty: "String".to_string(),
                default_value: None,
                validators: None,
                visible: None,
                is_secret: false,
            },
        ),
        (
            PAGINATION_INPUT_ARG_FIRST.to_string(),
            MetaInputValue {
                name: PAGINATION_INPUT_ARG_FIRST.to_string(),
                description: None,
                ty: "Int".to_string(),
                default_value: None,
                validators: None,
                visible: None,
                is_secret: false,
            },
        ),
        (
            PAGINATION_INPUT_ARG_LAST.to_string(),
            MetaInputValue {
                name: PAGINATION_INPUT_ARG_LAST.to_string(),
                description: None,
                ty: "Int".to_string(),
                default_value: None,
                validators: None,
                visible: None,
                is_secret: false,
            },
        ),
        (
            PAGINATION_INPUT_ARG_ORDER_BY.to_string(),
            MetaInputValue {
                name: PAGINATION_INPUT_ARG_ORDER_BY.to_string(),
                description: None,
                ty: Type::nullable(orderby_input_type).to_string(),
                default_value: None,
                visible: None,
                validators: None,
                is_secret: false,
            },
        ),
    ])
}

fn register_orderby_input(registry: &mut Registry, model_type_definition: &TypeDefinition) -> BaseType {
    let input_type_name = MetaNames::pagination_orderby_input(model_type_definition);
    registry.create_type(
        |registry| {
            let order_by_direction_type = register_dynaql_enum::<OrderByDirection>(registry);
            MetaType::InputObject {
                name: input_type_name.clone(),
                description: None,
                input_fields: IndexMap::from([(
                    RESERVED_FIELD_CREATED_AT.to_string(),
                    MetaInputValue {
                        name: RESERVED_FIELD_CREATED_AT.to_string(),
                        description: None,
                        ty: Type::nullable(order_by_direction_type).to_string(),
                        default_value: None,
                        visible: None,
                        validators: None,
                        is_secret: false,
                    },
                )]),
                oneof: true,
                visible: None,
                rust_typename: input_type_name.clone(),
            }
        },
        &input_type_name,
        &input_type_name,
    );
    BaseType::named(&input_type_name)
}
