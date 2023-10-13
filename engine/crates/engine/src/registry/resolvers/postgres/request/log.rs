use std::future::Future;

use common_types::LogEventType;
use postgres_types::transport::QueryResponse;
use runtime::log::LogEvent;

use crate::{registry::resolvers::postgres::context::PostgresContext, Error};

use super::RowData;

pub(super) async fn query<F>(
    ctx: &PostgresContext<'_>,
    sql: &str,
    operation: F,
) -> crate::Result<QueryResponse<RowData>>
where
    F: Future<Output = postgres_types::Result<QueryResponse<RowData>>>,
{
    let Some(log_endpoint_url) = ctx.fetch_log_endpoint_url()? else {
        return operation.await.map_err(|error| Error::new(error.to_string()));
    };

    let request_id = ctx.ray_id()?;
    let start_time = web_time::Instant::now();
    let response = operation.await;
    let duration = start_time.elapsed();

    let body = response
        .as_ref()
        .ok()
        .and_then(|response| serde_json::to_string(&response.clone_rows()).ok());

    let r#type = LogEventType::SqlQuery {
        successful: response.is_ok(),
        sql: sql.to_string(),
        duration,
        body,
    };

    let log_event = LogEvent { request_id, r#type };

    reqwest::Client::new()
        .post(format!("{log_endpoint_url}/log-event"))
        .json(&log_event)
        .send()
        .await?;

    response.map_err(|error| Error::new(error.to_string()))
}
