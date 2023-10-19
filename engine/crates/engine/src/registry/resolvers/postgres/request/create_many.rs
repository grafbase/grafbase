use futures_util::TryFutureExt;
use grafbase_sql_ast::renderer::{self, Renderer};

use super::{log, query, RowData};
use crate::registry::resolvers::{postgres::context::PostgresContext, ResolvedValue};

pub(crate) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, crate::Error> {
    let input = ctx.create_many_input()?;
    let (sql, params) = renderer::Postgres::build(query::insert::build(&ctx, input)?);

    if ctx.mutation_is_returning() {
        let operation = ctx.transport().parameterized_query(&sql, params).map_ok(|values| {
            values
                .into_iter()
                .map(|value| serde_json::from_value::<RowData>(value).expect("should deserialize to RowData"))
                .collect()
        });
        let response = log::query(&ctx, &sql, operation).await?;
        let rows: Vec<_> = response.into_iter().map(|row| row.root).collect();
        let row_count = rows.len();

        Ok(ResolvedValue::new(serde_json::json!({
            "returning": rows,
            "rowCount": row_count,
        })))
    } else {
        let operation = ctx.transport().parameterized_execute(&sql, params);
        let row_count = log::execute(&ctx, &sql, operation).await?;

        Ok(ResolvedValue::new(serde_json::json!({
            "rowCount": row_count,
        })))
    }
}
