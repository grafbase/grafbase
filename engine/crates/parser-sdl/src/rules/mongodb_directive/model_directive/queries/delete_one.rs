use common_types::auth::Operations;
use engine::registry::{
    resolvers::{
        atlas_data_api::{AtlasDataApiResolver, OperationType},
        Resolver,
    },
    MetaField, MetaInputValue,
};

use crate::{
    registry::names::{MetaNames, INPUT_ARG_BY},
    rules::{mongodb_directive::model_directive::create_type_context::CreateTypeContext, visitor::VisitorContext},
};

pub(super) fn create(
    visitor_ctx: &mut VisitorContext<'_>,
    create_ctx: &CreateTypeContext<'_>,
    filter_oneof_type: &str,
    output_type_name: &str,
) {
    let mutation_name = MetaNames::mutation_delete(create_ctx.r#type);

    let mut query = MetaField::new(mutation_name, output_type_name);
    query.description = Some(format!("Delete a unique {}", create_ctx.model_name()));

    let input_value = MetaInputValue::new(INPUT_ARG_BY, format!("{filter_oneof_type}!"));
    query.args = std::iter::once(input_value)
        .map(|input| (input.name.clone(), input))
        .collect();

    query.resolver = Resolver::MongoResolver(AtlasDataApiResolver {
        collection: create_ctx.collection().to_string(),
        operation_type: OperationType::DeleteOne,
        directive_name: create_ctx.config().name.clone(),
    });

    query.required_operation = Some(Operations::DELETE);
    query.auth = create_ctx.model_auth().cloned();

    visitor_ctx.push_namespaced_mutation(create_ctx.mutation_type_name(), query);
}
