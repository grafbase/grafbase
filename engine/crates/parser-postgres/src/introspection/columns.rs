use postgres_connector_types::{
    database_definition::{self, ColumnType, DatabaseDefinition, ScalarType, TableColumn},
    transport::{Transport, TransportExt},
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum IdentityGeneration {
    /// Cannot insert a custom value to the column, always generated.
    #[serde(rename = "ALWAYS")]
    Always,
    /// Can optionally insert a custom value to the column, by default generated.
    #[serde(rename = "BY DEFAULT")]
    ByDefault,
}

impl From<IdentityGeneration> for database_definition::IdentityGeneration {
    fn from(value: IdentityGeneration) -> Self {
        match value {
            IdentityGeneration::Always => Self::Always,
            IdentityGeneration::ByDefault => Self::ByDefault,
        }
    }
}

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
    identity_generation: Option<IdentityGeneration>,
}

pub(super) async fn introspect<T>(transport: &T, database_definition: &mut DatabaseDefinition) -> crate::Result<()>
where
    T: Transport + Sync,
{
    let query = include_str!("queries/columns.sql");

    let result = transport
        .collect_query::<Row>(query, vec![super::blocked_schemas()])
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

        if let Some(identity_generation) = row.identity_generation {
            column.set_identity_generation(identity_generation);
        }

        database_definition.push_table_column(column);
    }

    Ok(())
}
