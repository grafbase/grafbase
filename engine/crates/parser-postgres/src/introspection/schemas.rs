use postgres_types::{
    database_definition::DatabaseDefinition,
    transport::{Transport, TransportExt},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Row {
    name: String,
}

pub(super) async fn introspect<T>(transport: &T, database_definition: &mut DatabaseDefinition) -> crate::Result<()>
where
    T: Transport + Sync,
{
    let query = "SELECT nspname AS name FROM pg_namespace WHERE nspname <> ALL ($1) ORDER BY name";

    let result = transport
        .collect_query::<Row>(query, vec![super::blocked_schemas()])
        .await?;

    for row in result {
        database_definition.push_schema(row.name);
    }

    Ok(())
}
