use std::iter;

use common_types::auth::Operations;
use engine::registry::{
    resolvers::{
        atlas_data_api::{AtlasDataApiResolver, OperationType},
        Resolver,
    },
    MetaField, MetaInputValue,
};

use crate::{
    registry::names::{MetaNames, INPUT_ARG_INPUT},
    rules::{
        mongodb_directive::model_directive::{create_type_context::CreateTypeContext, types},
        visitor::VisitorContext,
    },
};

pub(super) fn create(
    visitor_ctx: &mut VisitorContext<'_>,
    create_ctx: &CreateTypeContext<'_>,
    create_input_type: &str,
) {
    let output_type_name = types::create::register_many_output(visitor_ctx, create_ctx);
    let query_name = MetaNames::mutation_create_many(create_ctx.r#type);

    let mut query = MetaField::new(query_name, output_type_name);
    query.description = Some(format!("Create multiple {}s", create_ctx.model_name()));

    let input_value = MetaInputValue::new(INPUT_ARG_INPUT, format!("[{create_input_type}!]!"));
    query.args = iter::once(input_value)
        .map(|input| (input.name.clone(), input))
        .collect();

    query.resolver = Resolver::MongoResolver(AtlasDataApiResolver {
        operation_type: OperationType::InsertMany,
        directive_name: create_ctx.config().name.clone(),
        collection: create_ctx.collection().to_string(),
    });

    query.required_operation = Some(Operations::CREATE);
    query.auth = create_ctx.model_auth().cloned();

    visitor_ctx.push_namespaced_mutation(create_ctx.mutation_type_name(), query);
}
