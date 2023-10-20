use std::borrow::Cow;

use engine::registry::MetaInputValue;
use postgres_types::database_definition::TableWalker;

use crate::registry::context::{InputContext, OutputContext};

pub(crate) fn register(input_ctx: &InputContext<'_>, table: TableWalker<'_>, output_ctx: &mut OutputContext) -> String {
    let input_type_name = input_ctx.update_type_name(table.client_name());

    output_ctx.with_input_type(&input_type_name, table.id(), |builder| {
        for column in table.columns() {
            let mut client_type: Cow<'static, str> = column
                .graphql_base_type()
                .expect("non-supported types are filtered before reaching this")
                .into();

            if column.database_type().is_json() {
                client_type = Cow::Borrowed("SimpleJSON");
            }

            let r#type = if column.is_array() {
                input_ctx.update_type_name(&format!("{client_type}Array"))
            } else {
                input_ctx.update_type_name(&client_type)
            };

            let mut input = MetaInputValue::new(column.client_name(), r#type);
            input.rename = Some(table.database_name().to_string());

            builder.push_input_column(input, column.id());
        }
    });

    input_type_name
}
