use futures::{pin_mut, stream::BoxStream, StreamExt};
use serde_json::Value;
use tokio_postgres::GenericClient;

use crate::error::Error;

pub(super) fn query<'a, T>(client: &'a T, query: &'a str, params: Vec<Value>) -> BoxStream<'a, Result<Value, Error>>
where
    T: GenericClient + Send + Sync,
{
    Box::pin(async_stream::try_stream! {
        let params = super::json_to_string(params);
        let row_stream = client.query_raw_txt(query, params).await?;

        pin_mut!(row_stream);

        while let Some(Ok(row)) = row_stream.next().await {
            yield serde_json::from_value(super::conversion::row_to_json(&row)?)?;
        }
    })
}

pub(super) async fn execute<T>(client: &T, query: &str, params: Vec<Value>) -> crate::Result<i64>
where
    T: GenericClient,
{
    let params = super::json_to_string(params);
    let row_stream = client.query_raw_txt(query, params).await?;

    pin_mut!(row_stream);

    while (row_stream.next().await).is_some() {}

    let command_tag = row_stream.command_tag().unwrap_or_default();
    let mut command_tag_split = command_tag.split(' ');
    let command_tag_name = command_tag_split.next().unwrap_or_default();

    let row_count = if command_tag_name == "INSERT" {
        // INSERT returns OID first and then number of rows
        command_tag_split.nth(1)
    } else {
        // other commands return number of rows (if any)
        command_tag_split.next()
    }
    .and_then(|s| s.parse::<i64>().ok());

    Ok(row_count.unwrap_or_default())
}
