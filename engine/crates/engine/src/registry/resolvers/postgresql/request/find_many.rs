use super::query::{self, SelectBuilder};
use crate::{
    registry::resolvers::{
        postgresql::context::{CollectionArgs, PostgresContext},
        ResolvedValue,
    },
    Error,
};
use grafbase_sql_ast::renderer::{self, Renderer};
use postgresql_types::transport::Transport;
use serde_json::Value;

#[derive(Debug, serde::Deserialize)]
struct Response {
    root: Value,
}

pub(crate) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, Error> {
    let mut builder = SelectBuilder::new(ctx.table(), ctx.collection_selection(), "root");

    builder.set_collection_args(CollectionArgs::new(&ctx, ctx.table(), &ctx.root_field()));

    if let Ok(filter) = ctx.filter() {
        builder.set_filter(filter);
    }

    let (sql, params) = renderer::Postgres::build(query::build(builder));

    let response = ctx
        .transport()
        .parameterized_query::<Response>(&sql, params)
        .await
        .map_err(|error| Error::new(error.to_string()))?;

    Ok(ResolvedValue::new(
        response
            .into_single_row()
            .map(|row| row.root)
            .unwrap_or(Value::Array(Vec::new())),
    ))
}
