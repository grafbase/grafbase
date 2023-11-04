mod columns;
mod enums;
mod foreign_keys;
mod schemas;
mod tables;
mod unique_constraints;

use std::sync::OnceLock;

use postgres_connector_types::{database_definition::DatabaseDefinition, transport::Transport};
use serde_json::Value;

/// A list of schemas to filter out automatically on every introspection.
static BLOCKED_SCHEMAS: &[&str] = &["pg_catalog", "pg_toast", "information_schema"];

/// Introspects a PostgreSQL database, creates a new registry with the corresponding GraphQL types.
/// Adds the introspected database definition to the registry to be used with the queries.
pub(crate) async fn introspect<T>(transport: &T) -> crate::Result<DatabaseDefinition>
where
    T: Transport + Sync,
{
    let mut database_definition = DatabaseDefinition::new(transport.connection_string());

    // order matters
    schemas::introspect(transport, &mut database_definition).await?;
    enums::introspect(transport, &mut database_definition).await?;
    tables::introspect(transport, &mut database_definition).await?;
    columns::introspect(transport, &mut database_definition).await?;
    foreign_keys::introspect(transport, &mut database_definition).await?;
    unique_constraints::introspect(transport, &mut database_definition).await?;

    database_definition.finalize();

    Ok(database_definition)
}

pub(super) fn blocked_schemas() -> Value {
    static SCHEMAS: OnceLock<Vec<Value>> = OnceLock::new();

    let result = SCHEMAS
        .get_or_init(|| {
            BLOCKED_SCHEMAS
                .iter()
                .map(|schema| (*schema).to_string().into())
                .collect()
        })
        .clone();

    Value::Array(result)
}
