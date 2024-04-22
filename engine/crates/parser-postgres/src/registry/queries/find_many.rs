use common_types::auth::Operations;
use indexmap::IndexMap;
use postgres_connector_types::database_definition::TableWalker;
use registry_v1::{
    resolvers::{
        postgres::{Operation, PostgresResolver},
        transformer::Transformer,
        Resolver,
    },
    MetaField, MetaInputValue,
};

use crate::registry::context::{InputContext, OutputContext};

pub(crate) fn register(
    input_ctx: &InputContext<'_>,
    table: TableWalker<'_>,
    filter_type: &str,
    output_ctx: &mut OutputContext,
) {
    let type_name = table.client_name();
    let field_name = input_ctx.collection_query_name(type_name);
    let output_type = input_ctx.connection_type_name(type_name);

    let mut field = MetaField::new(field_name, output_type.as_str());
    field.description = Some(format!("Paginated query to fetch the whole list of {type_name}"));

    field.args = IndexMap::from([
        ("filter".to_string(), MetaInputValue::new("filter", filter_type)),
        ("first".to_string(), MetaInputValue::new("first", "Int")),
        ("last".to_string(), MetaInputValue::new("last", "Int")),
        ("before".to_string(), MetaInputValue::new("before", "String")),
        ("after".to_string(), MetaInputValue::new("after", "String")),
    ]);

    let order_by_type = input_ctx.orderby_input_type_name(type_name);

    let order_by_value = MetaInputValue::new("orderBy", format!("[{order_by_type}]"));
    field.args.insert("orderBy".to_string(), order_by_value);

    field.resolver = Resolver::PostgresResolver(PostgresResolver::new(Operation::FindMany, input_ctx.directive_name()))
        .and_then(Transformer::PostgresPageInfo);

    field.required_operation = Some(Operations::LIST);

    output_ctx.push_query(field);
}
