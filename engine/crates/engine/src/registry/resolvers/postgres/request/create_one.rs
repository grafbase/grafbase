use grafbase_sql_ast::renderer::{self, Renderer};
use postgres_types::transport::Transport;
use serde_json::Value;

use crate::registry::resolvers::{
    postgres::{context::PostgresContext, request::query},
    ResolvedValue,
};

#[derive(Debug, serde::Deserialize)]
struct Response {
    root: Value,
}

pub(crate) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, crate::Error> {
    let (sql, params) = renderer::Postgres::build(query::insert::build(&ctx, [ctx.create_input()?])?);

    let response = ctx
        .transport()
        .parameterized_query::<Response>(&sql, params)
        .await
        .map_err(|error| crate::Error::new(error.to_string()))?;

    Ok(ResolvedValue::new(
        response.into_single_row().map(|row| row.root).unwrap_or(Value::Null),
    ))
}
