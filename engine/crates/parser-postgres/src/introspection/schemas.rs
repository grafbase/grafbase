use postgres_types::{database_definition::DatabaseDefinition, transport::Transport};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Row {
    name: String,
}

pub(super) async fn introspect<T>(transport: &T, database_definition: &mut DatabaseDefinition) -> crate::Result<()>
where
    T: Transport + Sync,
{
    let result = transport
        .parameterized_query::<Row>(
            "SELECT nspname AS name FROM pg_namespace WHERE nspname <> ALL ($1) ORDER BY name",
            vec![super::blocked_schemas()],
        )
        .await?;

    for row in result {
        database_definition.push_schema(row.name);
    }

    Ok(())
}
