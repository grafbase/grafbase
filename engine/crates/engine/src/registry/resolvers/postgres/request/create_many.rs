use super::query;
use crate::registry::resolvers::{postgres::context::PostgresContext, ResolvedValue};
use grafbase_sql_ast::renderer::{self, Renderer};
use postgres_types::transport::Transport;
use serde_json::Value;

#[derive(Debug, serde::Deserialize)]
struct Response {
    root: Value,
}

pub(crate) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, crate::Error> {
    let (sql, params) = renderer::Postgres::build(query::insert::build(&ctx, ctx.many_input()?)?);

    let response = ctx
        .transport()
        .parameterized_query::<Response>(&sql, params)
        .await
        .map_err(|error| crate::Error::new(error.to_string()))?;

    let rows = response.into_rows().map(|row| row.root).collect();

    Ok(ResolvedValue::new(Value::Array(rows)))
}
