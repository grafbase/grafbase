use engine::registry::MetaInputValue;
use postgresql_types::database_definition::TableWalker;

use crate::registry::context::{InputContext, OutputContext};

const LOGICAL_OPERATIONS: &[(&str, &str)] = &[
    ("ALL", "All of the filters must match"),
    ("NONE", "None of the filters must match"),
    ("ANY", "At least one of the filters must match"),
];

pub(crate) fn register(input_ctx: &InputContext<'_>, table: TableWalker<'_>, output_ctx: &mut OutputContext) -> String {
    let type_name = input_ctx.type_name(table.client_name());
    let input_type_name = format!("{type_name}Collection");

    output_ctx.with_input_type(&input_type_name, table.id(), |builder| {
        for column in table.columns() {
            let scalar = column
                .graphql_base_type()
                .expect("unsupported types are filtered out at this point");

            let type_name = if column.is_array() {
                input_ctx.filter_type_name(&format!("{scalar}Array"))
            } else {
                input_ctx.filter_type_name(&scalar)
            };

            let input = MetaInputValue::new(column.client_name(), type_name.as_ref())
                .with_rename(Some(column.database_name().to_string()));

            builder.push_input_column(input, column.id());
        }

        for relation in table.relations() {
            let mut type_name = format!(
                "{}Collection",
                input_ctx.type_name(relation.referenced_table().client_name())
            );

            if !relation.is_referenced_row_unique() {
                type_name = format!("{type_name}Contains");
            }

            let input = MetaInputValue::new(relation.client_field_name(), type_name.as_ref());

            builder.push_input_relation(input, relation.id());
        }

        for (name, description) in LOGICAL_OPERATIONS {
            let r#type = format!("[{input_type_name}]");
            let input = MetaInputValue::new(*name, r#type).with_description(*description);

            builder.push_non_mapped_input_column(input);
        }

        let contains_type_name = format!("{input_type_name}Contains");
        builder.with_input_type(&contains_type_name, table.id(), |builder| {
            let value = MetaInputValue::new("contains", input_type_name.as_ref());
            builder.push_non_mapped_input_column(value);
        });
    });

    input_type_name
}
