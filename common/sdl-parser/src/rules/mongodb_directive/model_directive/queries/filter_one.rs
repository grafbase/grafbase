use dynaql::{
    indexmap::IndexMap,
    registry::{
        resolvers::{
            atlas_data_api::{AtlasDataApiResolver, OperationType},
            Resolver,
        },
        Deprecation, MetaField, MetaInputValue,
    },
};
use grafbase::auth::Operations;

use crate::{
    registry::names::INPUT_ARG_BY,
    rules::{mongodb_directive::model_directive::CreateTypeContext, visitor::VisitorContext},
    utils::to_lower_camelcase,
};

pub(super) fn create(
    visitor_ctx: &mut VisitorContext<'_>,
    create_ctx: &CreateTypeContext<'_>,
    filter_oneof_type: &str,
) {
    let type_name = create_ctx.model_name();
    let query_name = to_lower_camelcase(type_name);
    let query_description = format!("Query a single {type_name} by a field");
    let filter_description = format!("The field and value by which to query the {type_name}");

    let filter_input =
        MetaInputValue::new(INPUT_ARG_BY, format!("{filter_oneof_type}!")).with_description(filter_description);

    let mut args = IndexMap::new();
    args.insert(INPUT_ARG_BY.to_string(), filter_input);

    let resolver = Resolver::MongoResolver(AtlasDataApiResolver {
        collection: create_ctx.collection().to_string(),
        operation_type: OperationType::FindOne,
        directive_name: create_ctx.config().name.clone(),
    });

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
