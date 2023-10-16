use grafbase_sql_ast::renderer::{self, Renderer};
use postgres_types::transport::Transport;
use serde_json::Value;

use crate::{
    registry::resolvers::{postgres::context::PostgresContext, ResolvedValue},
    Error,
};

use super::{log, query, RowData};

pub(crate) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, Error> {
    let (sql, params) = renderer::Postgres::build(query::delete::build(&ctx, ctx.by_filter()?)?);
    let operation = ctx.transport().parameterized_query::<RowData>(&sql, params);
    let rows = log::query(&ctx, &sql, operation).await?;

    Ok(ResolvedValue::new(
        rows.into_iter().next().map(|row| row.root).unwrap_or(Value::Null),
    ))
}
