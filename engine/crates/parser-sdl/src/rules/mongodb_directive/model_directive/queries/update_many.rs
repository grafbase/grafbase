use common_types::auth::Operations;
use engine::registry::{
    resolvers::{
        atlas_data_api::{AtlasDataApiResolver, OperationType},
        Resolver,
    },
    MetaField, MetaInputValue,
};

use crate::{
    registry::names::{MetaNames, INPUT_ARG_FILTER, INPUT_ARG_INPUT},
    rules::{mongodb_directive::CreateTypeContext, visitor::VisitorContext},
};

pub(crate) fn create(
    visitor_ctx: &mut VisitorContext<'_>,
    create_ctx: &CreateTypeContext<'_>,
    filter_input_type: &str,
    update_input_type: &str,
    update_output_type: &str,
) {
    let query_name = MetaNames::mutation_update_many(create_ctx.r#type);

    let mut mutation = MetaField::new(query_name, update_output_type);
    mutation.description = Some(format!("Update many {}s", create_ctx.model_name()));

    mutation.args.insert(
        INPUT_ARG_FILTER.to_string(),
        MetaInputValue::new(INPUT_ARG_FILTER, format!("{filter_input_type}!")),
    );

    mutation.args.insert(
        INPUT_ARG_INPUT.to_string(),
        MetaInputValue::new(INPUT_ARG_INPUT, format!("{update_input_type}!")),
    );

    mutation.resolver = Resolver::MongoResolver(AtlasDataApiResolver {
        operation_type: OperationType::UpdateMany,
        directive_name: create_ctx.config().name.clone(),
        collection: create_ctx.collection().to_string(),
    });

    mutation.required_operation = Some(Operations::UPDATE);
    mutation.auth = create_ctx.model_auth().cloned();

    visitor_ctx.push_namespaced_mutation(create_ctx.mutation_type_name(), mutation);
}
