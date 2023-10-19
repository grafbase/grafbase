use engine::futures_util::TryFutureExt;
use postgres_types::{
    database_definition::{ColumnType, DatabaseDefinition, ScalarType, TableColumn},
    transport::Transport,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Row {
    schema: String,
    table_name: String,
    column_name: String,
    type_id: u32,
    type_name: String,
    type_schema: String,
    is_array: bool,
    has_default: bool,
    is_nullable: bool,
}

pub(super) async fn introspect<T>(transport: &T, database_definition: &mut DatabaseDefinition) -> crate::Result<()>
where
    T: Transport + Sync,
{
    let query = include_str!("queries/columns.sql");

    let result: Vec<Row> = transport
        .parameterized_query(query, vec![super::blocked_schemas()])
        .map_ok(postgres_types::transport::map_result)
        .await?;

    for row in result {
        let Some(schema_id) = database_definition.get_schema_id(&row.schema) else {
            continue;
        };
        let Some(table_id) = database_definition.get_table_id(schema_id, &row.table_name) else {
            continue;
        };

        // If the type is an array, it's named `_type` in the database. We don't need that info in the type
        // name, we store enums without an underscore in our interner.
        let type_name = row.type_name.trim_start_matches('_');

        let enum_id = database_definition
            .get_schema_id(&row.type_schema)
            .and_then(|enum_schema_id| database_definition.get_enum_id(enum_schema_id, type_name));

        let database_type = match enum_id {
            Some(enum_id) => ColumnType::Enum(enum_id),
            None => ColumnType::Scalar(ScalarType::from(row.type_id)),
        };

        let mut column = TableColumn::new(table_id, row.column_name, database_type);

        column.set_nullable(row.is_nullable);
        column.set_has_default(row.has_default);
        column.set_is_array(row.is_array);

        database_definition.push_table_column(column);
    }

    Ok(())
}
