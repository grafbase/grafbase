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

    let result = transport
        .parameterized_query::<Row>(query, vec![super::blocked_schemas()])
        .await?;

    for row in result.into_rows() {
        let Some(schema_id) = database_definition.get_schema_id(&row.schema) else {
            continue;
        };
        let table = Table::<String>::new(schema_id, row.name);

        database_definition.push_table(table);
    }

    Ok(())
}
