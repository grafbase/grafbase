use crate::{
    registry::resolvers::{
        postgres::{
            context::PostgresContext,
            request::query::{self, SelectBuilder},
        },
        ResolvedValue,
    },
    Error,
};
use grafbase_sql_ast::renderer::{self, Renderer};
use postgres_types::transport::Transport;
use serde_json::Value;

#[derive(Debug, serde::Deserialize)]
struct Response {
    root: Value,
}

pub(super) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, Error> {
    let mut builder = SelectBuilder::new(ctx.table(), ctx.selection(), "root");

    if let Ok(filter) = ctx.by_filter() {
        builder.set_filter(filter);
    }

    let (sql, params) = renderer::Postgres::build(query::select::build(builder)?);

    let response = ctx
        .transport()
        .parameterized_query::<Response>(&sql, params)
        .await
        .map_err(|error| Error::new(error.to_string()))?;

    Ok(ResolvedValue::new(
        response.into_single_row().map(|row| row.root).unwrap_or(Value::Null),
    ))
}
