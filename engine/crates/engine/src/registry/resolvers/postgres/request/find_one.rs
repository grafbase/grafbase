use super::{log, RowData};
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

pub(super) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, Error> {
    let mut builder = SelectBuilder::new(ctx.table(), ctx.selection(), "root");

    if let Ok(filter) = ctx.by_filter() {
        builder.set_filter(filter);
    }

    let (sql, params) = renderer::Postgres::build(query::select::build(builder)?);

    let operation = ctx.transport().parameterized_query::<RowData>(&sql, params);
    let rows = log::query(&ctx, &sql, operation).await?;
    let row = rows.into_iter().next().map(|row| row.root).unwrap_or(Value::Null);

    Ok(ResolvedValue::new(row))
}
