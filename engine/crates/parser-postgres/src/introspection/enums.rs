use postgres_connector_types::{
    database_definition::{DatabaseDefinition, Enum, EnumVariant},
    transport::{Transport, TransportExt},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Row {
    schema: String,
    enum_name: String,
    enum_value: String,
}

pub(super) async fn introspect<T>(transport: &T, database_definition: &mut DatabaseDefinition) -> crate::Result<()>
where
    T: Transport + Sync,
{
    let query = include_str!("queries/enums.sql");

    let result = transport
        .collect_query::<Row>(query, vec![super::blocked_schemas()])
        .await?;

    for row in result {
        let Some(schema_id) = database_definition.get_schema_id(&row.schema) else {
            continue;
        };

        let enum_id = match database_definition.get_enum_id(schema_id, &row.enum_name) {
            Some(enum_id) => enum_id,
            None => database_definition.push_enum(Enum::new(schema_id, row.enum_name)),
        };

        database_definition.push_enum_variant(EnumVariant::new(enum_id, row.enum_value));
    }

    Ok(())
}
