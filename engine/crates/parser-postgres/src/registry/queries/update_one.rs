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

pub(crate) fn register(
    input_ctx: &InputContext<'_>,
    table: TableWalker<'_>,
    filter_oneof_type: &str,
    update_input_type: &str,
    output_ctx: &mut OutputContext,
) {
    let type_name = input_ctx.mutation_return_type_name(table.client_name());
    let query_name = format!("{}_Update", table.client_name()).to_camel_case();

    let by_value = MetaInputValue::new("by", format!("{filter_oneof_type}!"));

    let input_value = MetaInputValue::new("input", format!("{update_input_type}!"));
    let mut meta_field = MetaField::new(query_name, type_name);

    meta_field.description = Some(format!("Update a unique {}", table.client_name()));
    meta_field.required_operation = Some(Operations::UPDATE);
    meta_field.args = [("by".to_string(), by_value), ("input".to_string(), input_value)].into();

    meta_field.resolver =
        Resolver::PostgresResolver(PostgresResolver::new(Operation::UpdateOne, input_ctx.directive_name()));

    output_ctx.push_mutation(meta_field);
}
