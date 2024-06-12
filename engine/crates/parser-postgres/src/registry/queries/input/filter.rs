use postgres_connector_types::database_definition::{RelationWalker, TableColumnWalker, TableWalker};
use registry_v1::MetaInputValue;

use crate::registry::context::{InputContext, InputTypeBuilder, OutputContext};

const LOGICAL_OPERATIONS: &[(&str, &str)] = &[
    ("ALL", "All of the filters must match"),
    ("NONE", "None of the filters must match"),
    ("ANY", "At least one of the filters must match"),
];

pub(crate) fn register(
    input_ctx: &InputContext<'_>,
    table: TableWalker<'_>,
    output_ctx: &mut OutputContext,
) -> (String, String) {
    let type_name = input_ctx.type_name(table.client_name());
    let complex_input = format!("{type_name}Collection");

    output_ctx.with_input_type(&complex_input, table.id(), |builder| {
        for column in table.columns() {
            add_column(input_ctx, column, builder);
        }

        for relation in table.relations() {
            add_relation(input_ctx, relation, builder);
        }

        add_logical_operations(builder, &complex_input);

        let contains_type_name = format!("{complex_input}Contains");
        builder.with_input_type(&contains_type_name, table.id(), |builder| {
            let value = MetaInputValue::new("contains", complex_input.as_ref());
            builder.push_input_value(value);
        });
    });

    let simple_input = format!("{}Collection", input_ctx.mutation_return_type_name(table.client_name()));

    output_ctx.with_input_type(&simple_input, table.id(), |builder| {
        for column in table.columns() {
            add_column(input_ctx, column, builder);
        }

        add_logical_operations(builder, &simple_input);
    });

    (simple_input, complex_input)
}

fn add_logical_operations(builder: &mut InputTypeBuilder, input_type_name: &str) {
    for (name, description) in LOGICAL_OPERATIONS {
        let r#type = format!("[{input_type_name}]");
        let input = MetaInputValue::new(*name, r#type).with_description(*description);

        builder.push_input_value(input);
    }
}

fn add_relation(input_ctx: &InputContext<'_>, relation: RelationWalker<'_>, builder: &mut InputTypeBuilder) {
    let mut type_name = format!(
        "{}Collection",
        input_ctx.type_name(relation.referenced_table().client_name())
    );

    if !relation.is_other_side_one() {
        type_name = format!("{type_name}Contains");
    }

    let input = MetaInputValue::new(relation.client_field_name(), type_name.as_ref());

    builder.push_input_relation(input, relation.id());
}

fn add_column(input_ctx: &InputContext<'_>, column: TableColumnWalker<'_>, builder: &mut InputTypeBuilder) {
    let scalar = column
        .graphql_base_type(None)
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
