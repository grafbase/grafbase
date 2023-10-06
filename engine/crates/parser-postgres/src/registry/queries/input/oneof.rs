use crate::registry::context::{InputContext, OutputContext};
use engine::registry::MetaInputValue;
use inflector::Inflector;
use itertools::Itertools;
use postgres_types::database_definition::TableWalker;

pub(crate) fn register(input_ctx: &InputContext<'_>, table: TableWalker<'_>, output_ctx: &mut OutputContext) -> String {
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
                        let input_value = super::input_value_from_column(column, false);
                        builder.push_input_column(input_value, column.id());
                    }
                });

                let query_name = type_prefix.to_camel_case();

                builder.map_unique_constraint(&query_name, constraint.id());
                builder.push_input_value(MetaInputValue::new(query_name, input_type_name));
            } else if let Some(column) = constraint.columns().next() {
                let input_value = super::input_value_from_column(column.table_column(), true);
                builder.push_input_column(input_value, column.table_column().id());
            } else {
                continue;
            }
        }
    });

    input_type_name
}
