use bytes::Bytes;
use futures::Future;
use grafbase_telemetry::{
    gql_response_status::{GraphqlResponseStatus, SubgraphResponseStatus},
    span::{GqlRecorderSpanExt, GRAFBASE_TARGET},
};
use runtime::fetch::FetchRequest;
use tracing::Span;

use crate::{
    engine::RateLimitContext,
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

pub(super) async fn execute_subgraph_request<'ctx, 'a, R: Runtime>(
    ctx: ExecutionContext<'ctx, R>,
    span: Span,
    subgraph_name: &str,
    make_request: impl FnOnce() -> FetchRequest<'a> + Send,
    ingester: impl ResponseIngester,
) -> ExecutionResult<SubgraphResponse> {
    ctx.engine
        .runtime
        .rate_limiter()
        .limit(&RateLimitContext::Subgraph(subgraph_name))
        .await?;

    let mut request = make_request();
    request.headers = ctx
        .hooks()
        .on_subgraph_request(
            subgraph_name,
            http::Method::POST,
            request.url,
            std::mem::take(&mut request.headers),
        )
        .await?;

    request
        .headers
        .insert(http::header::ACCEPT, http::HeaderValue::from_static("application/json"));

    let fetch_response = ctx.engine.runtime.fetcher().post(request).await.inspect_err(|err| {
        span.record_subgraph_status(SubgraphResponseStatus::HttpError);
        tracing::error!(target: GRAFBASE_TARGET, "{err}");
    })?;

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

impl<T> ResponseIngester for T
where
    T: FnOnce(Bytes) -> Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError> + Send,
{
    async fn ingest(self, bytes: Bytes) -> Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError> {
        self(bytes)
    }
}
