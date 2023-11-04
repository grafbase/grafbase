use common_types::auth::Operations;
use engine::registry::{
    resolvers::{
        postgres::{Operation, PostgresResolver},
        Resolver,
    },
    MetaField, MetaInputValue,
};
use inflector::Inflector;
use postgres_connector_types::database_definition::TableWalker;

use crate::registry::context::{InputContext, OutputContext};

pub(crate) fn register(
    input_ctx: &InputContext<'_>,
    table: TableWalker<'_>,
    simple_filter: &str,
    output_ctx: &mut OutputContext,
) {
    let type_name = input_ctx.batch_mutation_return_type_name(table.client_name());
    let query_name = format!("{}_Delete_Many", table.client_name()).to_camel_case();

    let filter_description = format!("The filter definining which rows to delete from the {type_name} table");
    let filter_input = MetaInputValue::new("filter", format!("{simple_filter}!")).with_description(filter_description);

    let mut meta_field = MetaField::new(query_name, type_name);
    meta_field.description = Some(format!("Delete multiple rows of {} by a filter", table.client_name()));
    meta_field.args = [("filter".to_string(), filter_input)].into();
    meta_field.resolver =
        Resolver::PostgresResolver(PostgresResolver::new(Operation::DeleteMany, input_ctx.directive_name()));

    meta_field.required_operation = Some(Operations::DELETE);

    output_ctx.push_mutation(meta_field);
}
