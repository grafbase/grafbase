use grafbase_sql_ast::{
    ast::Order,
    renderer::{self, Renderer},
};
use postgres_connector_types::transport::TransportExt;
use serde_json::Value;

use super::{
    log,
    query::{self, SelectBuilder},
};
use crate::{
    registry::resolvers::{
        postgres::context::{CollectionArgs, PostgresContext},
        resolved_value::SelectionData,
        ResolvedValue,
    },
    Error,
};

pub(crate) async fn execute(ctx: PostgresContext<'_>) -> Result<ResolvedValue, Error> {
    let mut builder = SelectBuilder::new(ctx.table(), ctx.collection_selection(), "root");

    let args = CollectionArgs::new(&ctx.database_definition(), ctx.table(), &ctx.root_field())?;
    let mut selection_data = SelectionData::default();

    if let Some(first) = args.first() {
        selection_data.set_first(first);
    }

    if let Some(last) = args.last() {
        selection_data.set_last(last);
    }

    let explicit_order = args
        .order_by()
        .raw_order()
        .map(|(column, order)| {
            let order = order.map(|order| match order {
                Order::DescNullsFirst => "DESC",
                _ => "ASC",
            });

            (column.to_string(), order)
        })
        .collect();

    selection_data.set_order_by(explicit_order);
    builder.set_collection_args(args);

    if let Ok(filter) = ctx.filter() {
        builder.set_filter(filter);
    }

    let (sql, params) = renderer::Postgres::build(query::select::build(builder)?);
    let operation = ctx.transport().collect_query(&sql, params);
    let rows = log::query(&ctx, &sql, operation).await?;

    let response_data = rows
        .into_iter()
        .next()
        .map(|row| row.root)
        .unwrap_or(Value::Array(Vec::new()));

    let resolved_value = ResolvedValue::new(response_data).with_selection_data(selection_data);

    Ok(resolved_value)
}
