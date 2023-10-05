use grafbase_sql_ast::renderer::{self, Renderer};
use postgres_types::transport::Transport;
use serde_json::Value;

use crate::{
    registry::resolvers::{postgres::context::PostgresContext, ResolvedValue},
    Error,
};

use super::query;

#[derive(Debug, serde::Deserialize)]
struct Response {
    root: Value,
}

pub(crate) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, Error> {
    let (sql, params) = renderer::Postgres::build(query::delete::build(&ctx, ctx.filter()?)?);

    let response = ctx
        .transport()
        .parameterized_query::<Response>(&sql, params)
        .await
        .map_err(|error| Error::new(error.to_string()))?;

    let rows = response.into_rows().map(|row| row.root).collect();
    Ok(ResolvedValue::new(Value::Array(rows)))
}
