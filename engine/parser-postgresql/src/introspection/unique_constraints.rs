use postgresql_types::{
    database_definition::{DatabaseDefinition, UniqueConstraint, UniqueConstraintColumn},
    transport::Transport,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Row {
    schema: String,
    constraint_name: String,
    table_name: String,
    column_name: String,
}

pub(super) async fn introspect<T>(transport: &T, database_definition: &mut DatabaseDefinition) -> crate::Result<()>
where
    T: Transport + Sync,
{
    let query = include_str!("queries/unique_constraints.sql");

    let result = transport
        .parameterized_query::<Row>(query, vec![super::blocked_schemas()])
        .await?;

    for row in result.into_rows() {
        let Some(schema_id) = database_definition.get_schema_id(&row.schema) else { continue };
        let Some(table_id) = database_definition.get_table_id(schema_id, &row.table_name) else { continue };
        let Some(column_id) = database_definition.get_table_column_id(table_id, &row.column_name) else { continue };

        let constraint_id = match database_definition.get_unique_constraint_id(table_id, &row.constraint_name) {
            Some(id) => id,
            None => {
                let constraint = UniqueConstraint::new(table_id, row.constraint_name);
                database_definition.push_unique_constraint(constraint)
            }
        };

        let column = UniqueConstraintColumn::new(constraint_id, column_id);
        database_definition.push_unique_constraint_column(column);
    }

    Ok(())
}
