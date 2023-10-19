use futures_util::TryFutureExt;
use grafbase_sql_ast::renderer::{self, Renderer};
use serde_json::Value;

use super::log;
use crate::registry::resolvers::{
    postgres::{context::PostgresContext, request::query},
    ResolvedValue,
};

pub(crate) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, crate::Error> {
    let (sql, params) = renderer::Postgres::build(query::insert::build(&ctx, [ctx.create_input()?])?);

    if ctx.mutation_is_returning() {
        let operation = ctx
            .transport()
            .parameterized_query(&sql, params)
            .map_ok(postgres_types::transport::map_result);
        let rows = log::query(&ctx, &sql, operation).await?;
        let row = rows.into_iter().next().map(|row| row.root).unwrap_or(Value::Null);
        let row_count = if row.is_null() { 0 } else { 1 };

        Ok(ResolvedValue::new(serde_json::json!({
            "returning": row,
            "rowCount": row_count,
        })))
    } else {
        let operation = ctx.transport().parameterized_execute(&sql, params);
        let row_count = log::execute(&ctx, &sql, operation).await?;

        Ok(ResolvedValue::new(serde_json::json!({
            "rowCount": row_count
        })))
    }
}
