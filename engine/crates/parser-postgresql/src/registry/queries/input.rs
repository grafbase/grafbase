use std::borrow::Cow;

use engine::registry::MetaInputValue;
use inflector::Inflector;
use itertools::Itertools;
use postgresql_types::database_definition::{TableColumnWalker, TableWalker};

use crate::registry::context::{InputContext, OutputContext};

pub(super) fn register_oneof_filter(
    input_ctx: &InputContext<'_>,
    table: TableWalker<'_>,
    output_ctx: &mut OutputContext,
) -> String {
    let type_name = input_ctx.type_name(table.client_name());
    let input_type_name = format!("{type_name}ByInput");

    output_ctx.with_input_type(&input_type_name, table.id(), move |builder| {
        builder.oneof(true);

        for constraint in table.unique_constraints() {
            if constraint.columns().len() > 1 {
                let type_prefix = constraint
                    .columns()
                    .map(|column| column.table_column().client_name())
                    .join("_");

                let input_type_name = format!("{type_name}_{type_prefix}_Input").to_pascal_case();

                builder.with_input_type(&input_type_name, table.id(), move |builder| {
                    for column in constraint.columns() {
                        let column = column.table_column();
                        let input_value = input_value_from_column(column, false);
                        builder.push_input_column(input_value, column.id());
                    }
                });

                let query_name = type_prefix.to_camel_case();

                builder.push_input_value(MetaInputValue::new(query_name, input_type_name));
            } else if let Some(column) = constraint.columns().next() {
                let input_value = input_value_from_column(column.table_column(), true);
                builder.push_input_column(input_value, column.table_column().id());
            } else {
                continue;
            }
        }
    });

    input_type_name
}

fn input_value_from_column(column: TableColumnWalker<'_>, oneof: bool) -> MetaInputValue {
    let mut client_type = column
        .graphql_type()
        .expect("unsupported types are filtered out at this point");

    // Oneof types can't enforce arguments, the runtime expects one of the arguments to be
    // defined. For nested input types, we must enforce any argument that cannot be null.
    if !oneof && !column.nullable() {
        client_type = Cow::from(format!("{client_type}!"))
    }

    MetaInputValue::new(column.client_name(), client_type.as_ref())
        .with_rename(Some(column.database_name().to_string()))
}
