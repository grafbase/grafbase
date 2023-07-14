use std::iter;

use dynaql::registry::{
    resolvers::{
        atlas_data_api::{AtlasDataApiResolver, OperationType},
        Resolver,
    },
    MetaField, MetaInputValue,
};
use grafbase::auth::Operations;

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
    let output_type_name = types::register_create_output_type(visitor_ctx, create_ctx);
    let query_name = MetaNames::mutation_create(create_ctx.r#type);

    let mut query = MetaField::new(query_name, output_type_name);
    query.description = Some(format!("Create a {}", create_ctx.model_name()));

    let input_value = MetaInputValue::new(INPUT_ARG_INPUT, create_input_type);
    query.args = iter::once(input_value)
        .map(|input| (input.name.clone(), input))
        .collect();

    query.resolver = Resolver::MongoResolver(AtlasDataApiResolver {
        operation_type: OperationType::InsertOne,
        directive_name: create_ctx.config().name.clone(),
        collection: create_ctx.collection().to_string(),
    });

    query.required_operation = Some(Operations::CREATE);
    query.auth = create_ctx.model_auth().clone();

    visitor_ctx.mutations.push(query);
}
