use common_types::auth::Operations;
use engine::registry::{
    resolvers::{
        postgresql::{Operation, PostgresResolver},
        Resolver,
    },
    MetaField, MetaInputValue,
};
use inflector::Inflector;
use postgres_types::database_definition::TableWalker;

use crate::registry::context::{InputContext, OutputContext};

pub(crate) fn register(
    input_ctx: &InputContext<'_>,
    table: TableWalker<'_>,
    filter_oneof_type: &str,
    output_ctx: &mut OutputContext,
) {
    let type_name = input_ctx.delete_type_name(table.client_name());
    let query_name = format!("{}_Delete", table.client_name()).to_camel_case();

    let filter_description = format!("The field and value by which to delete the {type_name}");
    let filter_input = MetaInputValue::new("by", format!("{filter_oneof_type}!")).with_description(filter_description);

    let mut meta_field = MetaField::new(query_name, type_name.to_string());
    meta_field.description = Some(format!("Delete a unique {} by a field", table.client_name()));
    meta_field.args = [("by".to_string(), filter_input)].into();
    meta_field.resolver =
        Resolver::PostgresResolver(PostgresResolver::new(Operation::DeleteOne, input_ctx.directive_name()));

    meta_field.required_operation = Some(Operations::DELETE);

    output_ctx.push_mutation(meta_field);
}
