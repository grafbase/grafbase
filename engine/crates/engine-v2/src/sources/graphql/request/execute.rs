use std::borrow::Cow;

use bytes::Bytes;
use futures::Future;
use grafbase_telemetry::{
    gql_response_status::{GraphqlResponseStatus, SubgraphResponseStatus},
    span::{GqlRecorderSpanExt, GRAFBASE_TARGET},
};
use headers::HeaderMapExt;
use runtime::{
    bytes::OwnedOrSharedBytes,
    fetch::{FetchRequest, FetchResult, Fetcher},
    rate_limiting::RateLimitKey,
};
use schema::sources::graphql::{GraphqlEndpointId, GraphqlEndpointWalker};
use tower::retry::budget::Budget;
use tracing::Span;
use web_time::Duration;

use crate::{
    execution::{ExecutionContext, ExecutionError, ExecutionResult},
    response::SubgraphResponse,
    Runtime,
};

pub trait ResponseIngester: Send {
    fn ingest(
        self,
        bytes: OwnedOrSharedBytes,
    ) -> impl Future<Output = Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError>> + Send;
}

impl<T> ResponseIngester for T
where
    T: FnOnce(OwnedOrSharedBytes) -> Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError> + Send,
{
    async fn ingest(
        self,
        bytes: OwnedOrSharedBytes,
    ) -> Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError> {
        self(bytes)
    }
}

pub(crate) async fn execute_subgraph_request<'ctx, 'a, R: Runtime>(
    ctx: ExecutionContext<'ctx, R>,
    span: Span,
    endpoint_id: GraphqlEndpointId,
    retry_budget: Option<&Budget>,
    headers: http::HeaderMap,
    body: Bytes,
    ingester: impl ResponseIngester,
) -> ExecutionResult<SubgraphResponse> {
    let endpoint = ctx.schema().walk(endpoint_id);

    let request = {
        let mut headers = ctx
            .hooks()
            .on_subgraph_request(endpoint.subgraph_name(), http::Method::POST, endpoint.url(), headers)
            .await?;
        headers.typed_insert(headers::ContentType::json());
        headers.typed_insert(headers::ContentLength(body.len() as u64));
        headers.insert(http::header::ACCEPT, http::HeaderValue::from_static("application/json"));
        FetchRequest {
            url: Cow::Borrowed(endpoint.url()),
            headers,
            method: http::Method::POST,
            body,
            timeout: endpoint.timeout(),
        }
    };

    let response = retrying_fetch(ctx, endpoint, retry_budget, move || {
        ctx.engine.runtime.fetcher().fetch(request.clone())
    })
    .await?;

    tracing::debug!("{}", String::from_utf8_lossy(response.body()));

    let (status, response) = ingester.ingest(response.into_body()).await.inspect_err(|err| {
        let status = SubgraphResponseStatus::InvalidResponseError;
        span.record_subgraph_status(status);
        tracing::error!(target: GRAFBASE_TARGET, "{err}");
    })?;

    span.record_subgraph_status(SubgraphResponseStatus::GraphqlResponse(status));

    match response.subgraph_errors().next().map(|e| &e.message) {
        Some(error) => {
            tracing::error!(target: GRAFBASE_TARGET, "{error}");
        }
        None => {
            tracing::debug!(target: GRAFBASE_TARGET, "subgraph request")
        }
    }

    Ok(response)
}

pub(crate) async fn retrying_fetch<'ctx, R: Runtime, F, T>(
    ctx: ExecutionContext<'ctx, R>,
    endpoint: GraphqlEndpointWalker<'_>,
    retry_budget: Option<&Budget>,
    fetch: impl Fn() -> F + Send + Sync,
) -> ExecutionResult<T>
where
    F: Future<Output = FetchResult<T>> + Send,
    T: Send,
{
    let mut result = rate_limited_fetch(ctx, endpoint, &fetch).await;

    let Some(retry_budget) = retry_budget else {
        return result;
    };

    let mut counter = 0;

    loop {
        match result {
            Ok(bytes) => {
                retry_budget.deposit();
                return Ok(bytes);
            }
            Err(err) => {
                if retry_budget.withdraw().is_ok() {
                    let jitter = rand::random::<f64>() * 2.0;
                    let exp_backoff = (100 * 2u64.pow(counter)) as f64;
                    let backoff_ms = (exp_backoff * jitter).round() as u64;

                    ctx.engine.runtime.sleep(Duration::from_millis(backoff_ms)).await;

                    counter += 1;

                    result = rate_limited_fetch(ctx, endpoint, &fetch).await;
                } else {
                    return Err(err);
                }
            }
        }
    }
}

async fn rate_limited_fetch<'ctx, R: Runtime, F, T>(
    ctx: ExecutionContext<'ctx, R>,
    endpoint: GraphqlEndpointWalker<'ctx>,
    fetch: impl Fn() -> F + Send,
) -> ExecutionResult<T>
where
    F: Future<Output = FetchResult<T>> + Send,
    T: Send,
{
    ctx.engine
        .runtime
        .rate_limiter()
        .limit(&RateLimitKey::Subgraph(endpoint.subgraph_name().into()))
        .await?;

    fetch().await.map_err(|error| ExecutionError::Fetch {
        subgraph_name: endpoint.subgraph_name().to_string(),
        error,
    })
}
