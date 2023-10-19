use engine::futures_util::TryFutureExt;
use postgres_types::{
    database_definition::{DatabaseDefinition, ForeignKey, ForeignKeyColumn},
    transport::Transport,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Row {
    constraint_name: String,
    constrained_schema: String,
    constrained_table_name: String,
    constrained_column_name: String,
    referenced_schema: String,
    referenced_table_name: String,
    referenced_column_name: String,
}

pub(super) async fn introspect<T>(transport: &T, database_definition: &mut DatabaseDefinition) -> crate::Result<()>
where
    T: Transport + Sync,
{
    let query = include_str!("queries/foreign_keys.sql");

    let result: Vec<Row> = transport
        .parameterized_query(query, vec![super::blocked_schemas()])
        .map_ok(postgres_types::transport::map_result)
        .await?;

    #[allow(clippy::manual_let_else)] // sorry, but match looks better here
    for row in result {
        let constrained_schema_id = match database_definition.get_schema_id(&row.constrained_schema) {
            Some(id) => id,
            None => continue,
        };

        let constrained_table_id =
            match database_definition.get_table_id(constrained_schema_id, &row.constrained_table_name) {
                Some(id) => id,
                None => continue,
            };

        let constrained_column_id =
            match database_definition.get_table_column_id(constrained_table_id, &row.constrained_column_name) {
                Some(id) => id,
                None => continue,
            };

        let referenced_schema_id = match database_definition.get_schema_id(&row.referenced_schema) {
            Some(id) => id,
            None => continue,
        };

        let referenced_table_id =
            match database_definition.get_table_id(referenced_schema_id, &row.referenced_table_name) {
                Some(id) => id,
                None => continue,
            };

        let referenced_column_id =
            match database_definition.get_table_column_id(referenced_table_id, &row.referenced_column_name) {
                Some(id) => id,
                None => continue,
            };

        let foreign_key_id = match database_definition.get_foreign_key_id(constrained_schema_id, &row.constraint_name) {
            Some(id) => id,
            None => {
                let foreign_key = ForeignKey::new(
                    row.constraint_name,
                    constrained_schema_id,
                    constrained_table_id,
                    referenced_table_id,
                );

                database_definition.push_foreign_key(foreign_key)
            }
        };

        let column = ForeignKeyColumn::new(foreign_key_id, constrained_column_id, referenced_column_id);
        database_definition.push_foreign_key_column(column);
    }

    Ok(())
}
