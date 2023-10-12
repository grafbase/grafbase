use super::{log, query, RowData};
use crate::registry::resolvers::{postgres::context::PostgresContext, ResolvedValue};
use grafbase_sql_ast::renderer::{self, Renderer};
use postgres_types::transport::Transport;
use serde_json::Value;

pub(crate) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, crate::Error> {
    let (sql, params) = renderer::Postgres::build(query::insert::build(&ctx, ctx.create_many_input()?)?);

    let operation = ctx.transport().parameterized_query::<RowData>(&sql, params);
    let response = log::query(&ctx, &sql, operation).await?;
    let rows = response.into_rows().map(|row| row.root).collect();

    Ok(ResolvedValue::new(Value::Array(rows)))
}
