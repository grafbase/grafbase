use std::borrow::Cow;

use postgres_connector_types::database_definition::TableWalker;
use registry_v1::MetaInputValue;

use crate::registry::context::{InputContext, OutputContext};

pub(crate) fn register(input_ctx: &InputContext<'_>, table: TableWalker<'_>, output_ctx: &mut OutputContext) -> String {
    let input_type_name = input_ctx.create_input_name(table.client_name());

    output_ctx.with_input_type(&input_type_name, table.id(), move |builder| {
        for column in table.columns().filter(|column| column.allows_user_input()) {
            let r#type = column
                .graphql_type(input_ctx.namespace())
                .expect("non-supported types are filtered out at this point");

            let r#type = if column.nullable() || column.has_default() {
                r#type
            } else {
                Cow::Owned(format!("{type}!"))
            };

            let mut input = MetaInputValue::new(column.client_name(), r#type.to_string());
            input.rename = Some(table.database_name().to_string());

            builder.push_input_column(input, column.id());
        }
    });

    input_type_name
}
