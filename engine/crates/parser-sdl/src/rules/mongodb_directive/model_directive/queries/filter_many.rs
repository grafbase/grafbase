use common_types::auth::Operations;
use engine::{
    indexmap::IndexMap,
    names::{MONGODB_OUTPUT_FIELD_ID, OUTPUT_EDGE_CURSOR, OUTPUT_FIELD_ID},
    registry::{
        resolvers::{
            atlas_data_api::{AtlasDataApiResolver, OperationType},
            Resolver,
        },
        MetaField, MetaInputValue, NamedType, ObjectType, Registry,
    },
    AuthConfig,
};
use engine_parser::types::TypeDefinition;
use registry_v2::resolvers::transformer::Transformer;

use crate::{
    registry::{
        names::{
            MetaNames, INPUT_ARG_FILTER, PAGINATION_FIELD_EDGES, PAGINATION_FIELD_EDGE_CURSOR,
            PAGINATION_FIELD_EDGE_NODE, PAGINATION_FIELD_PAGE_INFO, PAGINATION_INPUT_ARG_AFTER,
            PAGINATION_INPUT_ARG_BEFORE, PAGINATION_INPUT_ARG_FIRST, PAGINATION_INPUT_ARG_LAST,
            PAGINATION_INPUT_ARG_ORDER_BY,
        },
        pagination::register_page_info_type,
    },
    rules::{
        cache_directive::CacheDirective,
        mongodb_directive::model_directive::{
            create_type_context::CreateTypeContext, types::filter::register_orderby_input,
        },
        visitor::VisitorContext,
    },
    type_names::TypeNameExt,
};

pub(super) fn create(visitor_ctx: &mut VisitorContext<'_>, create_ctx: &CreateTypeContext<'_>, filter_type: &str) {
    let type_name = create_ctx.model_name();
    let field_name = MetaNames::query_collection(create_ctx.r#type);

    let output_type = register_collection_type(
        visitor_ctx.registry.get_mut(),
        create_ctx.r#type,
        create_ctx.model_auth(),
    );

    let mut field = MetaField::new(field_name, output_type.as_str());

    field.description = Some(format!("Paginated query to fetch the whole list of {type_name}"));
    field.cache_control = create_ctx.model_cache().cloned();

    let mut args = input_args(filter_type);

    let extra_order_fields = std::iter::once((OUTPUT_FIELD_ID, MONGODB_OUTPUT_FIELD_ID));
    let order_by_type_name = register_orderby_input(visitor_ctx, create_ctx.object, type_name, extra_order_fields);
    let order_by_type_name = format!("[{order_by_type_name}]");

    args.insert(
        PAGINATION_INPUT_ARG_ORDER_BY.to_string(),
        MetaInputValue::new(PAGINATION_INPUT_ARG_ORDER_BY, order_by_type_name),
    );

    field.resolver = Resolver::MongoResolver(AtlasDataApiResolver {
        collection: create_ctx.collection().to_string(),
        operation_type: OperationType::FindMany,
        directive_name: create_ctx.config().name.clone(),
    });

    field.args = args;
    field.required_operation = Some(Operations::LIST);
    field.auth = create_ctx.model_auth().cloned();

    visitor_ctx.push_namespaced_query(create_ctx.query_type_name(), field);
}

#[allow(clippy::borrowed_box)]
fn register_collection_type(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&Box<AuthConfig>>,
) -> NamedType<'static> {
    let type_name = MetaNames::pagination_connection_type(model_type_definition);

    registry.create_type(
        |registry| {
            let edge_type = register_edge_type(registry, model_type_definition, model_auth);
            let page_info_type = register_page_info_type(registry);
            ObjectType::new(
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

#[allow(clippy::borrowed_box)] // Get to fuck clippy, I'm trying to work here
fn register_edge_type(
    registry: &mut Registry,
    model_type_definition: &TypeDefinition,
    model_auth: Option<&Box<AuthConfig>>,
) -> NamedType<'static> {
    let type_name = MetaNames::pagination_edge_type(model_type_definition);
    let model_name = NamedType::from(MetaNames::model(model_type_definition));

    let fields = [
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
            resolver: Transformer::select(OUTPUT_EDGE_CURSOR).into(),
            required_operation: Some(Operations::LIST),
            auth: model_auth.cloned(),
            ..Default::default()
        },
    ];

    let object_type = ObjectType::new(type_name.clone(), fields)
        .with_cache_control(CacheDirective::parse(&model_type_definition.directives));

    registry.create_type(|_| object_type.into(), &type_name, &type_name);
    type_name.into()
}

fn input_args(filter_type: &str) -> IndexMap<String, MetaInputValue> {
    IndexMap::from([
        (
            INPUT_ARG_FILTER.to_string(),
            MetaInputValue::new(INPUT_ARG_FILTER, filter_type),
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
            PAGINATION_INPUT_ARG_BEFORE.to_string(),
            MetaInputValue::new(PAGINATION_INPUT_ARG_BEFORE, "String"),
        ),
        (
            PAGINATION_INPUT_ARG_AFTER.to_string(),
            MetaInputValue::new(PAGINATION_INPUT_ARG_AFTER, "String"),
        ),
    ])
}
