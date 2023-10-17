use super::{log, RowData};
use grafbase_sql_ast::renderer::{self, Renderer};
use postgres_types::transport::Transport;

use crate::registry::resolvers::{
    postgres::{context::PostgresContext, request::query},
    ResolvedValue,
};

pub(crate) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, crate::Error> {
    let (sql, params) = renderer::Postgres::build(query::update::build(&ctx, ctx.filter()?)?);

    if ctx.mutation_is_returning() {
        let operation = ctx.transport().parameterized_query::<RowData>(&sql, params);
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
