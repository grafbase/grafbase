use common_types::auth::Operations;
use engine::registry::{
    resolvers::{
        postgres::{Operation, PostgresResolver},
        Resolver,
    },
    MetaField, MetaInputValue,
};
use inflector::Inflector;
use postgres_types::database_definition::TableWalker;

use crate::registry::context::{InputContext, OutputContext};

pub(super) fn register(
    input_ctx: &InputContext<'_>,
    table: TableWalker<'_>,
    filter_oneof_type: &str,
    output_ctx: &mut OutputContext,
) {
    let type_name = input_ctx.type_name(table.client_name());
    let query_name = table.client_name().to_camel_case();
    let filter_description = format!("The field and value by which to query the {type_name}");

    let filter_input = MetaInputValue::new("by", format!("{filter_oneof_type}!")).with_description(filter_description);

    let mut meta_field = MetaField::new(query_name, type_name.to_string());
    meta_field.description = Some(format!("Query a single {type_name} by a field"));
    meta_field.args = [("by".to_string(), filter_input)].into();
    meta_field.resolver =
        Resolver::PostgresResolver(PostgresResolver::new(Operation::FindOne, input_ctx.directive_name()));

    meta_field.required_operation = Some(Operations::GET);

    output_ctx.push_query(meta_field);
}
