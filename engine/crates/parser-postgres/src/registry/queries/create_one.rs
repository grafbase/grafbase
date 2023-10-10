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
    create_input_type: &str,
    output_ctx: &mut OutputContext,
) {
    let type_name = input_ctx.reduced_type_name(table.client_name());
    let query_name = format!("{}_Create", table.client_name()).to_camel_case();
    let input_value = MetaInputValue::new("input", format!("{create_input_type}!"));
    let mut meta_field = MetaField::new(query_name, type_name);

    meta_field.description = Some(format!("Create a {}", table.client_name()));
    meta_field.args = [("input".to_string(), input_value)].into();
    meta_field.required_operation = Some(Operations::CREATE);

    meta_field.resolver =
        Resolver::PostgresResolver(PostgresResolver::new(Operation::CreateOne, input_ctx.directive_name()));

    output_ctx.push_mutation(meta_field);
}
