use postgres_connector_types::{
    database_definition::{DatabaseDefinition, Table},
    transport::{Transport, TransportExt},
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
        .collect_query::<Row>(query, vec![super::blocked_schemas()])
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
