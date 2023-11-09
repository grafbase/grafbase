use common_types::auth::Operations;
use dynamodb::constant;
use engine::{
    indexmap::IndexMap,
    names::INPUT_FIELD_FILTER_IN,
    registry::{
        self,
        enums::OrderByDirection,
        relations::MetaRelation,
        resolvers::{dynamo_querying::DynamoResolver, transformer::Transformer, Resolver},
        variables::VariableResolveDefinition,
        InputObjectType, MetaField, MetaInputValue, NamedType, Registry,
    },
    AuthConfig,
};
use engine_parser::types::{Type, TypeDefinition};

use super::{
    names::{
        INPUT_ARG_FILTER, PAGINATION_INPUT_ARG_AFTER, PAGINATION_INPUT_ARG_BEFORE, PAGINATION_INPUT_ARG_FIRST,
        PAGINATION_INPUT_ARG_LAST, PAGINATION_INPUT_ARG_ORDER_BY,
    },
    register_engine_enum,
};
use crate::{
    registry::names::{
        MetaNames, PAGE_INFO_FIELD_END_CURSOR, PAGE_INFO_FIELD_HAS_NEXT_PAGE, PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE,
        PAGE_INFO_FIELD_START_CURSOR, PAGE_INFO_TYPE, PAGINATION_FIELD_EDGES, PAGINATION_FIELD_EDGE_CURSOR,
        PAGINATION_FIELD_EDGE_NODE, PAGINATION_FIELD_PAGE_INFO,
    },
    rules::{cache_directive::CacheDirective, model_directive::METADATA_FIELD_CREATED_AT, visitor::VisitorContext},
    type_names::TypeNameExt,
};

fn register_edge_type(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&AuthConfig>,
) -> NamedType<'static> {
    let type_name = MetaNames::pagination_edge_type(model_type_definition);
    let model_name = NamedType::from(MetaNames::model(model_type_definition));
    registry.create_type(
        |_| {
            registry::ObjectType::new(
                type_name.clone(),
                [
                    MetaField {
                        name: PAGINATION_FIELD_EDGE_NODE.to_string(),
                        ty: model_name.as_non_null().into(),
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGINATION_FIELD_EDGE_CURSOR.to_string(),
                        ty: "String!".into(),
                        resolver: Transformer::select(&MetaNames::model(model_type_definition))
                            .and_then(Transformer::DynamoSelect {
                                key: constant::SK.to_string(),
                            })
                            .and_then(Transformer::ConvertSkToCursor),
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        ..Default::default()
                    },
                ],
            )
            .with_cache_control(CacheDirective::parse(&model_type_definition.directives))
            .into()
        },
        &type_name,
        &type_name,
    );
    type_name.into()
}

pub(crate) fn register_page_info_type(registry: &mut Registry) -> NamedType<'static> {
    registry.create_type(
        |_| {
            registry::ObjectType::new(
                PAGE_INFO_TYPE.to_string(),
                [
                    MetaField {
                        name: PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE.to_string(),
                        ty: "Boolean!".into(),
                        resolver: Transformer::PaginationData.and_then(Transformer::select("has_previous_page")),
                        required_operation: Some(Operations::LIST),
                        // TODO: Auth should be propagated down during resolution from the parent
                        // type. PageInfo type is not specific to any data model, what matters is
                        // the model authorization of the model on which we iterate over.
                        auth: None,
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGE_INFO_FIELD_HAS_NEXT_PAGE.to_string(),
                        ty: "Boolean!".into(),
                        resolver: Transformer::PaginationData.and_then(Transformer::select("has_next_page")),
                        required_operation: Some(Operations::LIST),
                        auth: None,
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGE_INFO_FIELD_START_CURSOR.to_string(),
                        ty: "String".into(),
                        resolver: Transformer::PaginationData.and_then(Transformer::select("start_cursor")),
                        required_operation: Some(Operations::LIST),
                        auth: None,
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGE_INFO_FIELD_END_CURSOR.to_string(),
                        ty: "String".into(),
                        resolver: Transformer::PaginationData.and_then(Transformer::select("end_cursor")),
                        required_operation: Some(Operations::LIST),
                        auth: None,
                        ..Default::default()
                    },
                ],
            )
            .into()
        },
        PAGE_INFO_TYPE,
        PAGE_INFO_TYPE,
    );
    PAGE_INFO_TYPE.into()
}

pub fn register_connection_type(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    _connection_edges: Vec<String>,
    model_auth: Option<&AuthConfig>,
) -> NamedType<'static> {
    let type_name = MetaNames::pagination_connection_type(model_type_definition);

    registry.create_type(
        |registry| {
            let edge_type = register_edge_type(registry, model_type_definition, model_auth);
            let page_info_type = register_page_info_type(registry);
            registry::ObjectType::new(
                type_name.clone(),
                [
                    MetaField {
                        name: PAGINATION_FIELD_PAGE_INFO.to_string(),
                        description: Some("Information to aid in pagination".to_string()),
                        ty: page_info_type.as_non_null().into(),
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGINATION_FIELD_EDGES.to_string(),
                        // TODO: Should this be really nullable?
                        ty: edge_type.as_nullable().list().into(),
                        required_operation: Some(Operations::LIST),
                        auth: model_auth.cloned(),
                        ..Default::default()
                    },
                ],
            )
            .with_cache_control(CacheDirective::parse(&model_type_definition.directives))
            .into()
        },
        &type_name,
        &type_name,
    );

    type_name.into()
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
    let cache_control = CacheDirective::parse(&model_type_definition.directives);

    ctx.queries.push(MetaField {
        name: field.clone(),
        mapped_name: None,
        description: Some(format!("Paginated query to fetch the whole list of `{type_name}`.")),
        args: {
            let mut args = generate_pagination_args(ctx.registry.get_mut(), model_type_definition);
            args.insert(
                INPUT_ARG_FILTER.to_string(),
                MetaInputValue::new(
                    INPUT_ARG_FILTER,
                    register_collection_filter(ctx.registry.get_mut(), model_type_definition),
                ),
            );
            args
        },
        // TODO: Should this be really nullable?
        ty: connection_type.as_nullable().into(),
        deprecation: engine::registry::Deprecation::NoDeprecated,
        cache_control,
        external: false,
        provides: None,
        requires: None,
        r#override: None,
        visible: None,
        compute_complexity: None,
        edges: Vec::new(),
        relation: Some(MetaRelation::base_collection_relation(
            field,
            &Type::new(&type_name).expect("Shouldn't fail"),
        )),
        // Multiple entities
        resolver: Resolver::DynamoResolver(DynamoResolver::ListResultByTypePaginated {
            r#type: VariableResolveDefinition::debug_string(type_name.to_string()),
            first: VariableResolveDefinition::input_type_name(PAGINATION_INPUT_ARG_FIRST),
            after: VariableResolveDefinition::input_type_name(PAGINATION_INPUT_ARG_AFTER),
            before: VariableResolveDefinition::input_type_name(PAGINATION_INPUT_ARG_BEFORE),
            last: VariableResolveDefinition::input_type_name(PAGINATION_INPUT_ARG_LAST),
            order_by: Some(VariableResolveDefinition::input_type_name(
                PAGINATION_INPUT_ARG_ORDER_BY,
            )),
            filter: Some(VariableResolveDefinition::input_type_name(INPUT_ARG_FILTER)),
            nested: Box::new(None),
        }),
        required_operation: Some(Operations::LIST),
        auth: model_auth.cloned(),
        shareable: false,
        inaccessible: false,
        tags: vec![],
    });
}

fn register_collection_filter(registry: &mut Registry, model_type_definition: &TypeDefinition) -> String {
    let input_type_name = MetaNames::collection_filter_input(model_type_definition);
    registry.create_type(
        |registry| {
            InputObjectType::new(
                input_type_name.clone(),
                [MetaInputValue::new("id", register_scalar_filter(registry, "ID"))],
            )
            .into()
        },
        &input_type_name,
        &input_type_name,
    );

    input_type_name
}

fn register_scalar_filter(registry: &mut Registry, scalar: &str) -> String {
    let input_type_name = MetaNames::collection_scalar_filter_input(scalar);
    registry.create_type(
        |_| {
            InputObjectType::new(
                input_type_name.clone(),
                [MetaInputValue::new(INPUT_FIELD_FILTER_IN, format!("[{scalar}!]"))],
            )
            .into()
        },
        &input_type_name,
        &input_type_name,
    );
    input_type_name
}

pub fn generate_pagination_args(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
) -> IndexMap<String, MetaInputValue> {
    let orderby_input_type = register_orderby_input(registry, model_type_definition);
    IndexMap::from([
        (
            PAGINATION_INPUT_ARG_AFTER.to_string(),
            MetaInputValue::new(PAGINATION_INPUT_ARG_AFTER, "String"),
        ),
        (
            PAGINATION_INPUT_ARG_BEFORE.to_string(),
            MetaInputValue::new(PAGINATION_INPUT_ARG_BEFORE, "String"),
        ),
        (
            PAGINATION_INPUT_ARG_FIRST.to_string(),
            MetaInputValue::new(PAGINATION_INPUT_ARG_FIRST, "Int"),
        ),
        (
            PAGINATION_INPUT_ARG_LAST.to_string(),
            MetaInputValue::new(PAGINATION_INPUT_ARG_LAST, "Int"),
        ),
        (
            PAGINATION_INPUT_ARG_ORDER_BY.to_string(),
            MetaInputValue::new(PAGINATION_INPUT_ARG_ORDER_BY, orderby_input_type.as_nullable()),
        ),
    ])
}

fn register_orderby_input(registry: &mut Registry, model_type_definition: &TypeDefinition) -> NamedType<'static> {
    let input_type_name = MetaNames::pagination_orderby_input(model_type_definition);
    registry.create_type(
        |registry| {
            let order_by_direction_type = register_engine_enum::<OrderByDirection>(registry);
            InputObjectType::new(
                input_type_name.to_string(),
                [MetaInputValue::new(
                    METADATA_FIELD_CREATED_AT,
                    order_by_direction_type.as_nullable(),
                )],
            )
            .with_oneof(true)
            .into()
        },
        input_type_name.as_str(),
        input_type_name.as_str(),
    );
    input_type_name
}
