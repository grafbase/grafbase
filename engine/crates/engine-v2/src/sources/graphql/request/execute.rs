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
    hooks::{ExecutedSubgraphRequestBuilder, ResponseInfo, ResponseKind},
    rate_limiting::RateLimitKey,
};
use schema::sources::graphql::{GraphqlEndpointId, GraphqlEndpointWalker};
use tower::retry::budget::Budget;
use tracing::Span;
use web_time::{Duration, SystemTime};

use crate::{
    execution::{ExecutionContext, ExecutionError, ExecutionResult},
    response::SubgraphResponse,
    sources::graphql::record,
    Runtime,
};

pub trait ResponseIngester: Send {
    fn ingest(
        self,
        bytes: http::Response<OwnedOrSharedBytes>,
    ) -> impl Future<Output = Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError>> + Send;

    fn add_on_subgraph_response_data(&mut self, data: Vec<u8>) {}
}

pub(crate) struct SubgraphRequest<'ctx, 'a, R: Runtime> {
    pub ctx: ExecutionContext<'ctx, R>,
    pub span: Span,
    pub endpoint_id: GraphqlEndpointId,
    pub retry_budget: Option<&'a Budget>,
    pub headers: http::HeaderMap,
    pub body: Bytes,
}

pub(crate) async fn execute_subgraph_request<'ctx, 'a, R: Runtime>(
    SubgraphRequest {
        ctx,
        span,
        endpoint_id,
        retry_budget,
        headers,
        body,
    }: SubgraphRequest<'ctx, 'a, R>,
    request_info: &mut ExecutedSubgraphRequestBuilder<'_>,
    ingester: impl ResponseIngester,
) -> ExecutionResult<SubgraphResponse> {
    let endpoint = ctx.schema().walk(endpoint_id);

    let request = {
        let headers = ctx
            .hooks()
            .on_subgraph_request(endpoint.subgraph_name(), http::Method::POST, endpoint.url(), headers)
            .await
            .map_err(|error| {
                request_info.push_response(ResponseKind::HookError);
                error
            })?;

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

    let start = SystemTime::now();

    let response = retrying_fetch(ctx, endpoint, retry_budget, request_info, move || {
        record::subgraph_request_size(ctx, endpoint, request.body.len());
        ctx.engine.runtime.fetcher().fetch(request.clone())
    })
    .await;

    let duration = SystemTime::now()
        .duration_since(start)
        .unwrap_or(Duration::from_secs(0));

    let response = match response {
        Ok(response) => response,
        Err(e) => {
            let status = SubgraphResponseStatus::HttpError;

            request_info.set_graphql_status(subgraph_status);
            record::subgraph_duration(ctx, endpoint, status, None, duration);

            return Err(e);
        }
    };

    tracing::debug!("{}", String::from_utf8_lossy(response.body()));
    record::subgraph_response_size(ctx, endpoint, response.body().len());

    let status_code = response.status();

    let (status, response) = ingester.ingest(response).await.inspect_err(|err| {
        let subgraph_status = SubgraphResponseStatus::InvalidResponseError;
        request_info.set_graphql_status(subgraph_status);

        span.record_subgraph_status(subgraph_status);
        record::subgraph_duration(ctx, endpoint, subgraph_status, Some(status_code), duration);

        tracing::error!(target: GRAFBASE_TARGET, "{err}");
    })?;

    let subgraph_status = SubgraphResponseStatus::GraphqlResponse(status);
    request_info.set_graphql_status(subgraph_status);

    span.record_subgraph_status(subgraph_status);
    record::subgraph_duration(ctx, endpoint, subgraph_status, Some(status_code), duration);

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

async fn retrying_fetch<'ctx, R: Runtime, F, T>(
    ctx: ExecutionContext<'ctx, R>,
    endpoint: GraphqlEndpointWalker<'_>,
    retry_budget: Option<&Budget>,
    request_info: &mut ExecutedSubgraphRequestBuilder<'_>,
    fetch: impl Fn() -> F + Send + Sync,
) -> ExecutionResult<http::Response<T>>
where
    F: Future<Output = FetchResult<http::Response<T>>> + Send,
    T: Send,
{
    let mut result = rate_limited_fetch(ctx, endpoint, request_info, &fetch).await;

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
                    record::subgraph_retry(ctx, endpoint, false);

                    counter += 1;

                    result = rate_limited_fetch(ctx, endpoint, request_info, &fetch).await;
                } else {
                    record::subgraph_retry(ctx, endpoint, true);
                    return Err(err);
                }
            }
        }
    }
}

async fn rate_limited_fetch<'ctx, R: Runtime, F, T>(
    ctx: ExecutionContext<'ctx, R>,
    endpoint: GraphqlEndpointWalker<'ctx>,
    request_info: &mut ExecutedSubgraphRequestBuilder<'_>,
    fetch: impl Fn() -> F + Send,
) -> ExecutionResult<http::Response<T>>
where
    F: Future<Output = FetchResult<http::Response<T>>> + Send,
    T: Send,
{
    ctx.engine
        .runtime
        .rate_limiter()
        .limit(&RateLimitKey::Subgraph(endpoint.subgraph_name().into()))
        .await
        .map_err(|e| {
            request_info.push_response(ResponseKind::RateLimited);
            e
        })?;

    record::increment_inflight_requests(ctx, endpoint);
    let mut result = fetch().await;
    record::decrement_inflight_requests(ctx, endpoint);

    match result {
        Ok(ref mut response) => {
            if let Some(info) = response.extensions_mut().remove::<ResponseInfo>() {
                request_info.push_response(ResponseKind::Responsed(info));
            }
        }
        Err(_) => {
            request_info.push_response(ResponseKind::RequestError);
        }
    }

    result.map_err(|error| ExecutionError::Fetch {
        subgraph_name: endpoint.subgraph_name().to_string(),
        error,
    })
}
