use crate::{
    rules::{mongodb_directive::model_directive::CreateTypeContext, visitor::VisitorContext},
    utils::to_lower_camelcase,
};
use dynaql::{
    indexmap::IndexMap,
    registry::{
        resolvers::{
            atlas_data_api::{AtlasDataApiResolver, OperationType},
            transformer::Transformer,
            Resolver,
        },
        Deprecation, MetaField, MetaInputValue,
    },
};
use grafbase::auth::Operations;

pub(super) fn create(
    visitor_ctx: &mut VisitorContext<'_>,
    create_ctx: &CreateTypeContext<'_>,
    filter_oneof_type: &str,
) {
    let type_name = create_ctx.type_name();
    let query_name = to_lower_camelcase(type_name);
    let query_description = format!("Query a single {type_name} by a field");

    let filter_description = format!("The field and value by which to query the {type_name}");
    let filter_input = MetaInputValue::new("by", filter_oneof_type).with_description(filter_description);

    let mut args = IndexMap::new();
    args.insert("by".to_string(), filter_input);

    let directive = create_ctx.directive();

    let resolver = AtlasDataApiResolver {
        app_id: directive.app_id.clone(),
        api_key: directive.api_key.clone(),
        datasource: directive.data_source.clone(),
        database: directive.database.clone(),
        collection: create_ctx.collection().to_string(),
        operation_type: OperationType::FindOne,
    };

    let mongo_resolver = Resolver::MongoResolver(resolver);
    let local_key = Resolver::Transformer(Transformer::Select {
        key: "document".to_string(),
    });

    let resolver = Resolver::Composition(vec![mongo_resolver, local_key]);

    let meta_field = MetaField {
        name: query_name,
        description: Some(query_description),
        args,
        ty: type_name.into(),
        deprecation: Deprecation::NoDeprecated,
        cache_control: create_ctx.model_cache().clone(),
        resolver,
        required_operation: Some(Operations::GET),
        auth: create_ctx.model_auth().clone(),
        ..Default::default()
    };

    visitor_ctx.queries.push(meta_field);
}
