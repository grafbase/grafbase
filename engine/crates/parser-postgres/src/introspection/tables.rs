use engine::futures_util::TryStreamExt;
use postgres_types::{
    database_definition::{DatabaseDefinition, Table},
    transport::Transport,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Row {
    name: String,
    schema: String,
}

pub(super) async fn introspect<T>(transport: &T, database_definition: &mut DatabaseDefinition) -> crate::Result<()>
where
    T: Transport + Sync,
{
    let query = include_str!("queries/tables.sql");

    let result: Vec<Row> = transport
        .parameterized_query(query, vec![super::blocked_schemas()])
        .map_ok(postgres_types::transport::checked_map)
        .try_collect()
        .await?;

    for row in result {
        let Some(schema_id) = database_definition.get_schema_id(&row.schema) else {
            continue;
        };
        let table = Table::<String>::new(schema_id, row.name);

        database_definition.push_table(table);
    }

    Ok(())
}
