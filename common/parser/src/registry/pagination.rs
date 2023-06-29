use dynamodb::constant;
use dynaql::indexmap::IndexMap;
use dynaql::registry::enums::OrderByDirection;
use dynaql::registry::plan::{PaginationPage, SchemaPlan};
use dynaql::registry::relations::MetaRelation;
use dynaql::registry::transformers::Transformer;
use dynaql::registry::{self, InputObjectType, NamedType, Registry};
use dynaql::registry::{
    resolvers::context_data::ContextDataResolver, resolvers::dynamo_querying::DynamoResolver, resolvers::Resolver,
    resolvers::ResolverType, variables::VariableResolveDefinition, MetaField, MetaInputValue,
};
use dynaql::AuthConfig;
use dynaql_parser::types::{BaseType, Type, TypeDefinition};
use grafbase::auth::Operations;

use crate::registry::names::{
    MetaNames, PAGE_INFO_FIELD_END_CURSOR, PAGE_INFO_FIELD_HAS_NEXT_PAGE, PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE,
    PAGE_INFO_FIELD_START_CURSOR, PAGE_INFO_TYPE, PAGINATION_FIELD_EDGES, PAGINATION_FIELD_EDGE_CURSOR,
    PAGINATION_FIELD_EDGE_NODE, PAGINATION_FIELD_PAGE_INFO,
};
use crate::rules::cache_directive::CacheDirective;
use crate::rules::model_directive::METADATA_FIELD_CREATED_AT;
use crate::rules::visitor::VisitorContext;
use crate::type_names::TypeNameExt;

use super::names::{
    PAGINATION_INPUT_ARG_AFTER, PAGINATION_INPUT_ARG_BEFORE, PAGINATION_INPUT_ARG_FIRST, PAGINATION_INPUT_ARG_LAST,
    PAGINATION_INPUT_ARG_ORDER_BY,
};
use super::register_dynaql_enum;

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
                        plan: Some(
                            SchemaPlan::projection(vec!["id".to_string()], false)
                                .apply_cursor_encode(vec!["id".to_string()]),
                        ),
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

pub(super) fn register_page_info_type(registry: &mut Registry) -> NamedType<'static> {
    registry.create_type(
        |_| {
            registry::ObjectType::new(
                PAGE_INFO_TYPE.to_string(),
                [
                    MetaField {
                        name: PAGE_INFO_FIELD_HAS_PREVIOUS_PAGE.to_string(),
                        ty: "Boolean!".into(),
                        resolve: Some(Resolver {
                            id: None,
                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::PaginationData),
                        }),
                        plan: Some(SchemaPlan::pagination_page(PaginationPage::Previous)),
                        transformer: Some(Transformer::JSONSelect {
                            property: "has_previous_page".to_string(),
                        }),
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
                        resolve: Some(Resolver {
                            id: None,
                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::PaginationData),
                        }),
                        plan: Some(SchemaPlan::pagination_page(PaginationPage::Next)),
                        transformer: Some(Transformer::JSONSelect {
                            property: "has_next_page".to_string(),
                        }),
                        required_operation: Some(Operations::LIST),
                        auth: None,
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGE_INFO_FIELD_START_CURSOR.to_string(),
                        ty: "String".into(),
                        resolve: Some(Resolver {
                            id: None,
                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::PaginationData),
                        }),
                        plan: Some(
                            SchemaPlan::first(Some(SchemaPlan::projection(vec!["id".to_string()], false)))
                                .apply_cursor_encode(vec!["id".to_string()]),
                        ),
                        transformer: Some(Transformer::JSONSelect {
                            property: "start_cursor".to_string(),
                        }),
                        required_operation: Some(Operations::LIST),
                        auth: None,
                        ..Default::default()
                    },
                    MetaField {
                        name: PAGE_INFO_FIELD_END_CURSOR.to_string(),
                        ty: "String".into(),
                        resolve: Some(Resolver {
                            id: None,
                            r#type: ResolverType::ContextDataResolver(ContextDataResolver::PaginationData),
                        }),
                        plan: Some(
                            SchemaPlan::last(Some(SchemaPlan::projection(vec!["id".to_string()], false)))
                                .apply_cursor_encode(vec!["id".to_string()]),
                        ),
                        transformer: Some(Transformer::JSONSelect {
                            property: "end_cursor".to_string(),
                        }),
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

fn register_connection_type(
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

    let plan = Some(SchemaPlan::related(
        None,
        ctx.get_schema_id(&type_name),
        None,
        type_name.clone(),
    ));

    ctx.queries.push(MetaField {
        name: field.clone(),
        description: Some(format!("Paginated query to fetch the whole list of `{type_name}`.")),
        args: generate_pagination_args(ctx.registry.get_mut(), model_type_definition),
        // TODO: Should this be really nullable?
        ty: connection_type.as_nullable().into(),
        deprecation: dynaql::registry::Deprecation::NoDeprecated,
        cache_control,
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
            MetaInputValue::new(PAGINATION_INPUT_ARG_ORDER_BY, Type::nullable(orderby_input_type)),
        ),
    ])
}

fn register_orderby_input(registry: &mut Registry, model_type_definition: &TypeDefinition) -> BaseType {
    let input_type_name = MetaNames::pagination_orderby_input(model_type_definition);
    registry.create_type(
        |registry| {
            let order_by_direction_type = register_dynaql_enum::<OrderByDirection>(registry);
            InputObjectType::new(
                input_type_name.clone(),
                [MetaInputValue::new(
                    METADATA_FIELD_CREATED_AT,
                    Type::nullable(order_by_direction_type),
                )],
            )
            .with_oneof(true)
            .into()
        },
        &input_type_name,
        &input_type_name,
    );
    BaseType::named(&input_type_name)
}
