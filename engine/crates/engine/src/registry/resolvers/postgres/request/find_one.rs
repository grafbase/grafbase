use grafbase_sql_ast::renderer::{self, Renderer};
use postgres_types::transport::TransportExt;
use serde_json::Value;

use super::log;
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

pub(super) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, Error> {
    let mut builder = SelectBuilder::new(ctx.table(), ctx.selection(), "root");

    if let Ok(filter) = ctx.by_filter() {
        builder.set_filter(filter);
    }

    let (sql, params) = renderer::Postgres::build(query::select::build(builder)?);
    let operation = ctx.transport().collect_query(&sql, params);
    let rows = log::query(&ctx, &sql, operation).await?;
    let row = rows.into_iter().next().map(|row| row.root).unwrap_or(Value::Null);

    Ok(ResolvedValue::new(row))
}
