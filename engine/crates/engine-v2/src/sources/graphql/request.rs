use bytes::Bytes;
use futures::Future;
use grafbase_telemetry::{
    gql_response_status::{GraphqlResponseStatus, SubgraphResponseStatus},
    span::{GqlRecorderSpanExt, GRAFBASE_TARGET},
};
use runtime::{
    fetch::{FetchRequest, FetchResponse},
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
        bytes: Bytes,
    ) -> impl Future<Output = Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError>> + Send;
}

impl<T> ResponseIngester for T
where
    T: FnOnce(Bytes) -> Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError> + Send,
{
    async fn ingest(self, bytes: Bytes) -> Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError> {
        self(bytes)
    }
}

pub(super) async fn execute_subgraph_request<'ctx, 'a, R: Runtime>(
    ctx: ExecutionContext<'ctx, R>,
    span: Span,
    subgraph_id: GraphqlEndpointId,
    retry_budget: Option<&Budget>,
    make_request: impl FnOnce() -> FetchRequest<'a> + Send,
    ingester: impl ResponseIngester,
) -> ExecutionResult<SubgraphResponse> {
    let subgraph = ctx.schema().walk(subgraph_id);

    let mut request = make_request();
    request.headers = ctx
        .hooks()
        .on_subgraph_request(
            subgraph.name(),
            http::Method::POST,
            request.url,
            std::mem::take(&mut request.headers),
        )
        .await?;

    request
        .headers
        .insert(http::header::ACCEPT, http::HeaderValue::from_static("application/json"));

    let fetch_response = retrying_fetch(ctx, &request, subgraph_id, retry_budget).await?;

    tracing::debug!("{}", String::from_utf8_lossy(&fetch_response.bytes));

    let (status, response) = ingester.ingest(fetch_response.bytes).await.inspect_err(|err| {
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

async fn retrying_fetch<'ctx, R: Runtime>(
    ctx: ExecutionContext<'ctx, R>,
    request: &FetchRequest<'_>,
    subgraph_id: GraphqlEndpointId,
    retry_budget: Option<&Budget>,
) -> ExecutionResult<FetchResponse> {
    let subgraph = ctx.engine.schema.walk(subgraph_id);

    let mut result = rate_limited_fetch(ctx, subgraph, request).await;

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

                    result = rate_limited_fetch(ctx, subgraph, request).await;
                } else {
                    return Err(err);
                }
            }
        }
    }
}

async fn rate_limited_fetch<'ctx, R: Runtime>(
    ctx: ExecutionContext<'ctx, R>,
    subgraph: GraphqlEndpointWalker<'ctx>,
    request: &FetchRequest<'_>,
) -> ExecutionResult<FetchResponse> {
    ctx.engine
        .runtime
        .rate_limiter()
        .limit(&RateLimitKey::Subgraph(subgraph.name().into()))
        .await?;

    ctx.engine
        .runtime
        .fetcher()
        .post(request)
        .await
        .map_err(|error| ExecutionError::Fetch {
            subgraph_name: subgraph.name().to_string(),
            error,
        })
}
