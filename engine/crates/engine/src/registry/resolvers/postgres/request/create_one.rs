use grafbase_sql_ast::renderer::{self, Renderer};
use postgres_types::transport::Transport;
use serde_json::Value;

use crate::registry::resolvers::{
    postgres::{context::PostgresContext, request::query},
    ResolvedValue,
};

use super::{log, RowData};

pub(crate) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, crate::Error> {
    let (sql, params) = renderer::Postgres::build(query::insert::build(&ctx, [ctx.create_input()?])?);

    let operation = ctx.transport().parameterized_query::<RowData>(&sql, params);
    let response = log::query(&ctx, &sql, operation).await?;
    let row = response.into_single_row().map(|row| row.root).unwrap_or(Value::Null);

    Ok(ResolvedValue::new(row))
}
