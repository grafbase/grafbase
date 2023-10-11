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
    update_filter_type: &str,
    update_input_type: &str,
    output_ctx: &mut OutputContext,
) {
    let type_name = input_ctx.reduced_type_name(table.client_name());
    let query_name = format!("{}_Update_Many", table.client_name()).to_camel_case();

    let by_value = MetaInputValue::new("filter", format!("{update_filter_type}!"));

    let input_value = MetaInputValue::new("input", format!("{update_input_type}!"));
    let mut meta_field = MetaField::new(query_name, format!("[{type_name}]!"));

    meta_field.description = Some(format!("Update multiple {}s", table.client_name()));
    meta_field.required_operation = Some(Operations::UPDATE);
    meta_field.args = [("filter".to_string(), by_value), ("input".to_string(), input_value)].into();

    meta_field.resolver =
        Resolver::PostgresResolver(PostgresResolver::new(Operation::UpdateMany, input_ctx.directive_name()));

    output_ctx.push_mutation(meta_field);
}
